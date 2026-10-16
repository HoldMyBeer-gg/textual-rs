//! Full Pac-Man demo for textual-rs.
//!
//! Four ghosts with authentic AI: Blinky chases directly, Pinky targets 4 tiles
//! ahead, Inky uses a Blinky-relative vector, Clyde scatters when close.
//! Global scatter/chase cycling, frightened mode, eyes-return, lives, score.
//!
//! Run with: cargo run --example pacman
//! Controls:  Arrow keys / WASD to move   R: restart   Q / Ctrl+C: quit

use std::cell::{Cell, RefCell};
use std::time::Duration;

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};

use textual_rs::event::keybinding::KeyBinding;
use textual_rs::reactive::Reactive;
use textual_rs::widget::context::AppContext;
use textual_rs::widget::{EventPropagation, WidgetId};
use textual_rs::{App, Widget, WorkerProgress};

use crossterm::event::{KeyCode, KeyModifiers};

// ─── Constants ───────────────────────────────────────────────────────────────

const MAZE_W: usize = 28;
const MAZE_H: usize = 31;
const TUNNEL_ROW: i16 = 14;
const TICK_MS: u64 = 180;

const GHOST_HOME_COL: i16 = 13;
const GHOST_EXIT_ROW: i16 = 11;

const PAC_START: (i16, i16) = (13, 23);

// Global mode durations in ticks (alternating scatter / chase).
// Even indices = scatter, odd indices = chase. Last entry is chase-forever.
const MODE_DURATIONS: &[u32] = &[42, 120, 42, 120, 30, 120, 30, u32::MAX];

const FRIGHTENED_TICKS: u32 = 30;
const FRIGHTENED_FLASH_AT: u32 = 10;

// ─── Tile ────────────────────────────────────────────────────────────────────

#[derive(Clone, Copy, PartialEq, Debug)]
enum Tile {
    Wall,
    Dot,
    PowerDot,
    Empty,
    GhostDoor,
}

// ─── Maze ────────────────────────────────────────────────────────────────────

const MAZE_SRC: [&str; MAZE_H] = [
    "############################",
    "#............##............#",
    "#.####.#####.##.#####.####.#",
    "#o####.#####.##.#####.####o#",
    "#.####.#####.##.#####.####.#",
    "#..........................#",
    "#.####.##.########.##.####.#",
    "#.####.##.########.##.####.#",
    "#......##....##....##......#",
    "######.#####    #####.######",
    "######.#####    #####.######",
    "######.##          ##.######",
    "######.## ###--### ##.######",
    "######.## #      # ##.######",
    "      .   #      #   .      ",
    "######.## #      # ##.######",
    "######.## ######## ##.######",
    "######.##          ##.######",
    "######.## ######## ##.######",
    "######.## ######## ##.######",
    "#............##............#",
    "#.####.#####.##.#####.####.#",
    "#.####.#####.##.#####.####.#",
    "#o..##................##..o#",
    "###.##.##.########.##.##.###",
    "###.##.##.########.##.##.###",
    "#......##....##....##......#",
    "#.##########.##.##########.#",
    "#.##########.##.##########.#",
    "#..........................#",
    "############################",
];

fn parse_maze() -> ([[Tile; MAZE_W]; MAZE_H], usize) {
    let mut maze = [[Tile::Empty; MAZE_W]; MAZE_H];
    let mut count = 0usize;
    for (row, line) in MAZE_SRC.iter().enumerate() {
        for (col, ch) in line.chars().enumerate() {
            if col >= MAZE_W {
                break;
            }
            maze[row][col] = match ch {
                '#' => Tile::Wall,
                '.' => {
                    count += 1;
                    Tile::Dot
                }
                'o' => {
                    count += 1;
                    Tile::PowerDot
                }
                '-' => Tile::GhostDoor,
                _ => Tile::Empty,
            };
        }
    }
    (maze, count)
}

// ─── Ghost ───────────────────────────────────────────────────────────────────

#[derive(Clone, Copy, PartialEq, Debug)]
enum GhostKind {
    Blinky,
    Pinky,
    Inky,
    Clyde,
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum GhostMode {
    InHouse,
    LeavingHouse,
    Scatter,
    Chase,
    Frightened,
    Eyes,
}

#[derive(Clone, Debug, PartialEq)]
struct Ghost {
    kind: GhostKind,
    pos: (i16, i16),
    dir: (i8, i8),
    mode: GhostMode,
    move_timer: u8,
    leave_delay: u32,
    bounce_dir: i8, // horizontal bounce direction while in house
}

impl Ghost {
    fn new(kind: GhostKind) -> Self {
        let (pos, leave_delay, bounce_dir, mode) = match kind {
            GhostKind::Blinky => ((GHOST_HOME_COL, GHOST_EXIT_ROW), 0u32, 1i8, GhostMode::Scatter),
            GhostKind::Pinky => ((GHOST_HOME_COL, 14), 15, 1, GhostMode::InHouse),
            GhostKind::Inky => ((GHOST_HOME_COL - 2, 14), 35, -1, GhostMode::InHouse),
            GhostKind::Clyde => ((GHOST_HOME_COL + 2, 14), 55, 1, GhostMode::InHouse),
        };
        Ghost {
            kind,
            pos,
            dir: (1, 0),
            mode,
            move_timer: 0,
            leave_delay,
            bounce_dir,
        }
    }

    fn color(&self) -> Color {
        match self.kind {
            GhostKind::Blinky => Color::Red,
            GhostKind::Pinky => Color::Rgb(255, 183, 197),
            GhostKind::Inky => Color::Cyan,
            GhostKind::Clyde => Color::Rgb(255, 165, 0),
        }
    }

    fn scatter_corner(&self) -> (i16, i16) {
        match self.kind {
            GhostKind::Blinky => (25, 0),
            GhostKind::Pinky => (2, 0),
            GhostKind::Inky => (27, 30),
            GhostKind::Clyde => (0, 30),
        }
    }
}

// ─── Game State ──────────────────────────────────────────────────────────────

#[derive(Clone, Copy, PartialEq, Debug)]
enum GamePhase {
    Ready(u8),
    Playing,
    Dying(u8),
    GameOver,
    Win,
}

#[derive(Clone, Debug, PartialEq)]
struct GameState {
    phase: GamePhase,
    tick: u64,
    score: u32,
    high_score: u32,
    lives: u8,

    pac_pos: (i16, i16),
    pac_dir: (i8, i8),
    pac_next_dir: (i8, i8),
    pac_anim: u8,

    ghosts: [Ghost; 4],

    maze: [[Tile; MAZE_W]; MAZE_H],
    pellets_remaining: usize,

    global_mode: GhostMode,
    mode_index: usize,
    mode_ticks: u32,

    frightened_ticks: u32,
    ghost_eat_score: u32,
}

impl GameState {
    fn new() -> Self {
        let (maze, pellets) = parse_maze();
        GameState {
            phase: GamePhase::Ready(20),
            tick: 0,
            score: 0,
            high_score: 0,
            lives: 3,

            pac_pos: PAC_START,
            pac_dir: (-1, 0),
            pac_next_dir: (-1, 0),
            pac_anim: 0,

            ghosts: [
                Ghost::new(GhostKind::Blinky),
                Ghost::new(GhostKind::Pinky),
                Ghost::new(GhostKind::Inky),
                Ghost::new(GhostKind::Clyde),
            ],

            maze,
            pellets_remaining: pellets,

            global_mode: GhostMode::Scatter,
            mode_index: 0,
            mode_ticks: MODE_DURATIONS[0],

            frightened_ticks: 0,
            ghost_eat_score: 200,
        }
    }

    fn respawn(&mut self) {
        self.pac_pos = PAC_START;
        self.pac_dir = (-1, 0);
        self.pac_next_dir = (-1, 0);
        self.pac_anim = 0;

        self.ghosts = [
            Ghost::new(GhostKind::Blinky),
            Ghost::new(GhostKind::Pinky),
            Ghost::new(GhostKind::Inky),
            Ghost::new(GhostKind::Clyde),
        ];

        self.global_mode = GhostMode::Scatter;
        self.mode_index = 0;
        self.mode_ticks = MODE_DURATIONS[0];
        self.frightened_ticks = 0;
        self.ghost_eat_score = 200;
    }
}

// ─── Game Logic ──────────────────────────────────────────────────────────────

fn maze_tile(maze: &[[Tile; MAZE_W]; MAZE_H], x: i16, y: i16) -> Tile {
    if y < 0 || y >= MAZE_H as i16 {
        return Tile::Wall;
    }
    let x = if y == TUNNEL_ROW {
        ((x % MAZE_W as i16) + MAZE_W as i16) as usize % MAZE_W
    } else if x < 0 || x >= MAZE_W as i16 {
        return Tile::Wall;
    } else {
        x as usize
    };
    maze[y as usize][x]
}

fn wrap_pos(x: i16, y: i16) -> (i16, i16) {
    if y == TUNNEL_ROW {
        let x = ((x % MAZE_W as i16) + MAZE_W as i16) % MAZE_W as i16;
        (x, y)
    } else {
        (x, y)
    }
}

fn pac_passable(maze: &[[Tile; MAZE_W]; MAZE_H], x: i16, y: i16) -> bool {
    let t = maze_tile(maze, x, y);
    t != Tile::Wall && t != Tile::GhostDoor
}

fn ghost_passable(maze: &[[Tile; MAZE_W]; MAZE_H], x: i16, y: i16, mode: GhostMode) -> bool {
    let t = maze_tile(maze, x, y);
    if t == Tile::Wall {
        return false;
    }
    match mode {
        GhostMode::Eyes | GhostMode::LeavingHouse => true,
        _ => t != Tile::GhostDoor,
    }
}

fn sq_dist(a: (i16, i16), b: (i16, i16)) -> i32 {
    let dx = (a.0 - b.0) as i32;
    let dy = (a.1 - b.1) as i32;
    dx * dx + dy * dy
}

fn pseudo_rand(tick: u64, salt: u64) -> u64 {
    tick.wrapping_mul(0x9e3779b97f4a7c15)
        .wrapping_add(salt.wrapping_mul(0x6c62272e07bb0142))
        .rotate_left(30)
        .wrapping_mul(0xbf58476d1ce4e5b9)
}

// Classic priority order: up, left, right, down.
const DIRS: [(i8, i8); 4] = [(0, -1), (-1, 0), (1, 0), (0, 1)];

fn choose_dir(
    maze: &[[Tile; MAZE_W]; MAZE_H],
    pos: (i16, i16),
    current_dir: (i8, i8),
    target: (i16, i16),
    mode: GhostMode,
    tick: u64,
    salt: u64,
) -> (i8, i8) {
    let reverse = (-current_dir.0, -current_dir.1);

    if mode == GhostMode::Frightened {
        let valid: Vec<(i8, i8)> = DIRS
            .iter()
            .copied()
            .filter(|&d| {
                d != reverse
                    && ghost_passable(
                        maze,
                        pos.0 + d.0 as i16,
                        pos.1 + d.1 as i16,
                        mode,
                    )
            })
            .collect();
        if valid.is_empty() {
            return reverse;
        }
        return valid[pseudo_rand(tick, salt) as usize % valid.len()];
    }

    let mut best = reverse;
    let mut best_dist = i32::MAX;
    for &d in &DIRS {
        if d == reverse {
            continue;
        }
        let nx = pos.0 + d.0 as i16;
        let ny = pos.1 + d.1 as i16;
        if !ghost_passable(maze, nx, ny, mode) {
            continue;
        }
        let dist = sq_dist((nx, ny), target);
        if dist < best_dist {
            best_dist = dist;
            best = d;
        }
    }
    best
}

fn chase_target(
    ghost: &Ghost,
    pac_pos: (i16, i16),
    pac_dir: (i8, i8),
    blinky_pos: (i16, i16),
) -> (i16, i16) {
    match ghost.kind {
        GhostKind::Blinky => pac_pos,
        GhostKind::Pinky => (
            pac_pos.0 + 4 * pac_dir.0 as i16,
            pac_pos.1 + 4 * pac_dir.1 as i16,
        ),
        GhostKind::Inky => {
            let pivot = (
                pac_pos.0 + 2 * pac_dir.0 as i16,
                pac_pos.1 + 2 * pac_dir.1 as i16,
            );
            (
                2 * pivot.0 - blinky_pos.0,
                2 * pivot.1 - blinky_pos.1,
            )
        }
        GhostKind::Clyde => {
            if sq_dist(ghost.pos, pac_pos) > 64 {
                pac_pos
            } else {
                ghost.scatter_corner()
            }
        }
    }
}

fn advance_game(s: &mut GameState) {
    // Phase gating
    match s.phase {
        GamePhase::Ready(n) => {
            s.phase = if n == 0 {
                GamePhase::Playing
            } else {
                GamePhase::Ready(n - 1)
            };
            return;
        }
        GamePhase::Dying(n) => {
            if n == 0 {
                if s.lives <= 1 {
                    s.lives = 0;
                    s.phase = GamePhase::GameOver;
                } else {
                    s.lives -= 1;
                    s.respawn();
                    s.phase = GamePhase::Ready(20);
                }
            } else {
                s.phase = GamePhase::Dying(n - 1);
            }
            return;
        }
        GamePhase::Playing => {}
        _ => return,
    }

    s.tick += 1;

    // Global mode cycling
    if s.frightened_ticks > 0 {
        s.frightened_ticks -= 1;
        if s.frightened_ticks == 0 {
            // Restore frightened ghosts to current global mode
            for g in &mut s.ghosts {
                if g.mode == GhostMode::Frightened {
                    g.mode = s.global_mode;
                    g.dir = (-g.dir.0, -g.dir.1);
                }
            }
            s.ghost_eat_score = 200;
        }
    } else if s.mode_ticks > 0 {
        s.mode_ticks -= 1;
    } else {
        let new_idx = s.mode_index + 1;
        if new_idx < MODE_DURATIONS.len() {
            let new_mode = if new_idx % 2 == 0 {
                GhostMode::Scatter
            } else {
                GhostMode::Chase
            };
            let old_mode = s.global_mode;
            s.global_mode = new_mode;
            s.mode_index = new_idx;
            s.mode_ticks = MODE_DURATIONS[new_idx];
            if old_mode != new_mode {
                for g in &mut s.ghosts {
                    if g.mode == old_mode {
                        g.mode = new_mode;
                        g.dir = (-g.dir.0, -g.dir.1);
                    }
                }
            }
        }
    }

    // Move Pac-Man — try buffered direction first, fall back to current
    let try_next = (
        s.pac_pos.0 + s.pac_next_dir.0 as i16,
        s.pac_pos.1 + s.pac_next_dir.1 as i16,
    );
    if pac_passable(&s.maze, try_next.0, try_next.1) {
        s.pac_dir = s.pac_next_dir;
        s.pac_pos = wrap_pos(try_next.0, try_next.1);
    } else {
        let try_cur = (
            s.pac_pos.0 + s.pac_dir.0 as i16,
            s.pac_pos.1 + s.pac_dir.1 as i16,
        );
        if pac_passable(&s.maze, try_cur.0, try_cur.1) {
            s.pac_pos = wrap_pos(try_cur.0, try_cur.1);
        }
    }

    // Eat pellets
    let (px, py) = (s.pac_pos.0 as usize, s.pac_pos.1 as usize);
    match s.maze[py][px] {
        Tile::Dot => {
            s.maze[py][px] = Tile::Empty;
            s.score += 10;
            s.pellets_remaining -= 1;
        }
        Tile::PowerDot => {
            s.maze[py][px] = Tile::Empty;
            s.score += 50;
            s.pellets_remaining -= 1;
            s.frightened_ticks = FRIGHTENED_TICKS;
            s.ghost_eat_score = 200;
            for g in &mut s.ghosts {
                if matches!(g.mode, GhostMode::Chase | GhostMode::Scatter) {
                    g.mode = GhostMode::Frightened;
                    g.dir = (-g.dir.0, -g.dir.1);
                }
            }
        }
        _ => {}
    }

    s.pac_anim = (s.pac_anim + 1) % 8;

    if s.pellets_remaining == 0 {
        s.high_score = s.high_score.max(s.score);
        s.phase = GamePhase::Win;
        return;
    }

    // Move ghosts
    let blinky_pos = s.ghosts[0].pos;
    let pac_pos = s.pac_pos;
    let pac_dir = s.pac_dir;
    let global_mode = s.global_mode;
    let tick = s.tick;

    for i in 0..4 {
        match s.ghosts[i].mode {
            GhostMode::InHouse => {
                if s.ghosts[i].leave_delay > 0 {
                    s.ghosts[i].leave_delay -= 1;
                    // Bounce horizontally inside house
                    let nx = s.ghosts[i].pos.0 + s.ghosts[i].bounce_dir as i16;
                    let t = maze_tile(&s.maze, nx, s.ghosts[i].pos.1);
                    if t == Tile::Wall {
                        s.ghosts[i].bounce_dir *= -1;
                    } else {
                        s.ghosts[i].pos.0 = nx;
                    }
                } else {
                    s.ghosts[i].mode = GhostMode::LeavingHouse;
                }
                continue;
            }
            GhostMode::LeavingHouse => {
                let (gx, gy) = s.ghosts[i].pos;
                if gx != GHOST_HOME_COL {
                    s.ghosts[i].pos.0 += if gx < GHOST_HOME_COL { 1 } else { -1 };
                } else if gy > GHOST_EXIT_ROW {
                    s.ghosts[i].pos.1 -= 1;
                } else {
                    s.ghosts[i].mode = global_mode;
                    s.ghosts[i].dir = (-1, 0);
                }
                continue;
            }
            _ => {}
        }

        // Move timer
        if s.ghosts[i].move_timer > 0 {
            s.ghosts[i].move_timer -= 1;
            continue;
        }
        s.ghosts[i].move_timer = match s.ghosts[i].mode {
            GhostMode::Eyes => 0,
            GhostMode::Frightened => 2,
            _ => 1,
        };

        let target = if s.ghosts[i].mode == GhostMode::Eyes {
            (GHOST_HOME_COL, 14)
        } else if s.ghosts[i].mode == GhostMode::Scatter {
            s.ghosts[i].scatter_corner()
        } else if s.ghosts[i].mode == GhostMode::Chase {
            chase_target(&s.ghosts[i], pac_pos, pac_dir, blinky_pos)
        } else {
            (0, 0) // frightened uses random; target unused
        };

        let dir = choose_dir(
            &s.maze,
            s.ghosts[i].pos,
            s.ghosts[i].dir,
            target,
            s.ghosts[i].mode,
            tick,
            i as u64,
        );
        s.ghosts[i].dir = dir;
        let new_pos = wrap_pos(
            s.ghosts[i].pos.0 + dir.0 as i16,
            s.ghosts[i].pos.1 + dir.1 as i16,
        );
        s.ghosts[i].pos = new_pos;

        // Eyes arrive home
        if s.ghosts[i].mode == GhostMode::Eyes && s.ghosts[i].pos == (GHOST_HOME_COL, 14) {
            s.ghosts[i].mode = GhostMode::InHouse;
            s.ghosts[i].leave_delay = 25;
        }
    }

    // Collision
    for g in &mut s.ghosts {
        if g.pos != pac_pos {
            continue;
        }
        match g.mode {
            GhostMode::Frightened => {
                s.score += s.ghost_eat_score;
                s.ghost_eat_score = (s.ghost_eat_score * 2).min(1600);
                g.mode = GhostMode::Eyes;
                g.dir = (0, -1);
            }
            GhostMode::Chase | GhostMode::Scatter => {
                s.high_score = s.high_score.max(s.score);
                s.phase = GamePhase::Dying(16);
                return;
            }
            _ => {}
        }
    }

    s.high_score = s.high_score.max(s.score);
}

// ─── Sprites ─────────────────────────────────────────────────────────────────

fn pac_sprite(s: &GameState) -> (&'static str, Color) {
    if let GamePhase::Dying(n) = s.phase {
        return match n {
            13..=16 => ("●", Color::Yellow),
            9..=12 => ("◉", Color::Yellow),
            5..=8 => ("·", Color::Yellow),
            _ => (" ", Color::Yellow),
        };
    }
    if s.pac_anim % 4 < 2 {
        let sym = match s.pac_dir {
            (1, 0) => ">",
            (-1, 0) => "<",
            (0, -1) => "^",
            _ => "v",
        };
        (sym, Color::Yellow)
    } else {
        ("●", Color::Yellow)
    }
}

fn ghost_sprite(g: &Ghost, tick: u64, frightened_ticks: u32) -> (&'static str, Color) {
    match g.mode {
        GhostMode::Eyes => ("◌", Color::White),
        GhostMode::Frightened => {
            let flash = frightened_ticks <= FRIGHTENED_FLASH_AT && (tick / 3) % 2 == 0;
            let color = if flash {
                Color::White
            } else {
                Color::Rgb(0, 0, 200)
            };
            ("▲", color)
        }
        _ => ("▲", g.color()),
    }
}

// ─── Widget ──────────────────────────────────────────────────────────────────

struct PacManGame {
    state: Reactive<GameState>,
    own_id: Cell<Option<WidgetId>>,
    loop_handle: RefCell<Option<tokio::task::AbortHandle>>,
}

impl PacManGame {
    fn new() -> Self {
        Self {
            state: Reactive::new(GameState::new()),
            own_id: Cell::new(None),
            loop_handle: RefCell::new(None),
        }
    }

    fn spawn_loop(&self, ctx: &AppContext) {
        let Some(id) = self.own_id.get() else { return };
        let handle = ctx.run_worker_with_progress(id, |tx| {
            Box::pin(async move {
                let mut iv = tokio::time::interval(Duration::from_millis(TICK_MS));
                loop {
                    iv.tick().await;
                    if tx.send(()).is_err() {
                        break;
                    }
                }
            })
        });
        *self.loop_handle.borrow_mut() = Some(handle);
    }

    fn restart(&self, ctx: &AppContext) {
        if let Some(h) = self.loop_handle.borrow_mut().take() {
            h.abort();
        }
        let high = self.state.get_untracked().high_score;
        let mut fresh = GameState::new();
        fresh.high_score = high;
        self.state.set(fresh);
        self.spawn_loop(ctx);
    }
}

static BINDINGS: &[KeyBinding] = &[
    KeyBinding { key: KeyCode::Up,           modifiers: KeyModifiers::NONE, action: "up",      description: "Up",      show: false },
    KeyBinding { key: KeyCode::Down,         modifiers: KeyModifiers::NONE, action: "down",    description: "Down",    show: false },
    KeyBinding { key: KeyCode::Left,         modifiers: KeyModifiers::NONE, action: "left",    description: "Left",    show: false },
    KeyBinding { key: KeyCode::Right,        modifiers: KeyModifiers::NONE, action: "right",   description: "Right",   show: false },
    KeyBinding { key: KeyCode::Char('w'),    modifiers: KeyModifiers::NONE, action: "up",      description: "Up",      show: false },
    KeyBinding { key: KeyCode::Char('s'),    modifiers: KeyModifiers::NONE, action: "down",    description: "Down",    show: false },
    KeyBinding { key: KeyCode::Char('a'),    modifiers: KeyModifiers::NONE, action: "left",    description: "Left",    show: false },
    KeyBinding { key: KeyCode::Char('d'),    modifiers: KeyModifiers::NONE, action: "right",   description: "Right",   show: false },
    KeyBinding { key: KeyCode::Char('r'),    modifiers: KeyModifiers::NONE, action: "restart", description: "Restart", show: true  },
];

impl Widget for PacManGame {
    fn widget_type_name(&self) -> &'static str {
        "PacManGame"
    }

    fn can_focus(&self) -> bool {
        true
    }

    fn on_mount(&self, id: WidgetId) {
        self.own_id.set(Some(id));
    }

    fn on_unmount(&self, _id: WidgetId) {
        if let Some(h) = self.loop_handle.borrow_mut().take() {
            h.abort();
        }
        self.own_id.set(None);
    }

    fn key_bindings(&self) -> &[KeyBinding] {
        BINDINGS
    }

    fn on_action(&self, action: &str, ctx: &AppContext) {
        match action {
            "restart" => self.restart(ctx),
            dir => {
                let d: (i8, i8) = match dir {
                    "up" => (0, -1),
                    "down" => (0, 1),
                    "left" => (-1, 0),
                    "right" => (1, 0),
                    _ => return,
                };
                self.state.update(|s| s.pac_next_dir = d);
            }
        }
    }

    fn on_event(&self, event: &dyn std::any::Any, _ctx: &AppContext) -> EventPropagation {
        if event.downcast_ref::<WorkerProgress<()>>().is_some() {
            self.state.update(|s| advance_game(s));
            return EventPropagation::Stop;
        }
        EventPropagation::Continue
    }

    fn render(&self, ctx: &AppContext, area: Rect, buf: &mut Buffer) {
        // Lazy-start game loop on first render
        if self.loop_handle.borrow().is_none() {
            self.spawn_loop(ctx);
        }

        let s = self.state.get_untracked();

        // Black out the whole area
        for y in area.top()..area.bottom() {
            for x in area.left()..area.right() {
                if let Some(cell) = buf.cell_mut((x, y)) {
                    cell.set_symbol(" ");
                    cell.set_bg(Color::Black);
                    cell.set_fg(Color::Black);
                }
            }
        }

        // Center the game within the available area
        let total_h = MAZE_H as u16 + 2; // score row + maze + hint row
        let maze_x = area.x + area.width.saturating_sub(MAZE_W as u16) / 2;
        let maze_y = area.y + area.height.saturating_sub(total_h) / 2;

        // Score / lives bar
        let lives_str: String = "● ".repeat(s.lives as usize);
        let score_str = format!(
            "SCORE:{:06}  HI:{:06}  LIVES:{}",
            s.score,
            s.high_score,
            lives_str.trim_end(),
        );
        if maze_y < area.bottom() {
            buf.set_string(maze_x, maze_y, &score_str, Style::default().fg(Color::White));
        }

        let game_y = maze_y + 1;

        // Maze tiles
        for row in 0..MAZE_H {
            for col in 0..MAZE_W {
                let x = maze_x + col as u16;
                let y = game_y + row as u16;
                if x >= area.right() || y >= area.bottom() {
                    continue;
                }
                let (sym, fg, bg) = match s.maze[row][col] {
                    Tile::Wall => ("█", Color::Rgb(33, 33, 255), Color::Black),
                    Tile::GhostDoor => ("─", Color::Rgb(255, 183, 197), Color::Black),
                    Tile::Dot => ("·", Color::Rgb(255, 183, 197), Color::Black),
                    Tile::PowerDot => {
                        if (s.tick / 6) % 2 == 0 {
                            ("●", Color::White, Color::Black)
                        } else {
                            (" ", Color::Black, Color::Black)
                        }
                    }
                    Tile::Empty => (" ", Color::Black, Color::Black),
                };
                if let Some(cell) = buf.cell_mut((x, y)) {
                    cell.set_symbol(sym);
                    cell.set_fg(fg);
                    cell.set_bg(bg);
                }
            }
        }

        // Ghosts (drawn before Pac-Man so Pac-Man renders on top)
        for g in &s.ghosts {
            let (gx, gy) = g.pos;
            if gx < 0 || gy < 0 || gx >= MAZE_W as i16 || gy >= MAZE_H as i16 {
                continue;
            }
            let x = maze_x + gx as u16;
            let y = game_y + gy as u16;
            if x >= area.right() || y >= area.bottom() {
                continue;
            }
            let (sym, color) = ghost_sprite(g, s.tick, s.frightened_ticks);
            if let Some(cell) = buf.cell_mut((x, y)) {
                cell.set_symbol(sym);
                cell.set_fg(color);
                cell.set_bg(Color::Black);
            }
        }

        // Pac-Man
        if !matches!(s.phase, GamePhase::GameOver) {
            let (px, py) = s.pac_pos;
            if px >= 0 && py >= 0 && px < MAZE_W as i16 && py < MAZE_H as i16 {
                let x = maze_x + px as u16;
                let y = game_y + py as u16;
                if x < area.right() && y < area.bottom() {
                    let (sym, color) = pac_sprite(&s);
                    if let Some(cell) = buf.cell_mut((x, y)) {
                        cell.set_symbol(sym);
                        cell.set_fg(color);
                        cell.set_bg(Color::Black);
                    }
                }
            }
        }

        // Overlays — clipped to buffer bounds
        let mid_y = game_y + MAZE_H as u16 / 2;
        match s.phase {
            GamePhase::Ready(_) => {
                let y = game_y + 17;
                if y < area.bottom() {
                    let msg = " READY! ";
                    let x = maze_x + (MAZE_W as u16).saturating_sub(msg.len() as u16) / 2;
                    buf.set_string(x, y, msg, Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD));
                }
            }
            GamePhase::Win => {
                if mid_y < area.bottom() {
                    let msg = " YOU WIN!  Press R ";
                    let x = maze_x + (MAZE_W as u16).saturating_sub(msg.len() as u16) / 2;
                    buf.set_string(x, mid_y, msg, Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD));
                }
            }
            GamePhase::GameOver => {
                if mid_y < area.bottom() {
                    let msg = " GAME OVER  Press R ";
                    let x = maze_x + (MAZE_W as u16).saturating_sub(msg.len() as u16) / 2;
                    buf.set_string(x, mid_y, msg, Style::default().fg(Color::Red).add_modifier(Modifier::BOLD));
                }
            }
            _ => {}
        }

        // Hint row
        let hint_y = game_y + MAZE_H as u16;
        if hint_y < area.bottom() {
            buf.set_string(
                maze_x,
                hint_y,
                "← → ↑ ↓ / WASD: steer   R: restart   Q: quit",
                Style::default().fg(Color::DarkGray),
            );
        }
    }
}

// ─── Root Screen ─────────────────────────────────────────────────────────────

struct PacManScreen;

impl Widget for PacManScreen {
    fn widget_type_name(&self) -> &'static str {
        "PacManScreen"
    }

    fn compose(&self) -> Vec<Box<dyn Widget>> {
        vec![Box::new(PacManGame::new())]
    }

    fn render(&self, _ctx: &AppContext, _area: Rect, _buf: &mut Buffer) {}
}

// ─── Main ────────────────────────────────────────────────────────────────────

const CSS: &str = r#"
PacManScreen {
    background: $background;
    layout-direction: vertical;
}
PacManGame {
    flex-grow: 1;
    background: #000000;
}
"#;

fn main() -> anyhow::Result<()> {
    App::new(|| Box::new(PacManScreen))
        .with_css(CSS)
        .run()
}
