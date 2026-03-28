//! Core application runtime: event loop, widget tree management, and rendering.

use anyhow::Result;
use crossterm::event::{Event, EventStream, KeyCode, KeyEventKind, KeyModifiers};
use futures::StreamExt;
use ratatui::backend::{Backend, CrosstermBackend, TestBackend};
use ratatui::{Frame, Terminal};
use reactive_graph::owner::Owner;
use tokio::runtime::Builder;
use tokio::task;
use tokio::task::LocalSet;

use crate::css::cascade::{apply_cascade_to_tree, Stylesheet};

/// Built-in default CSS for all framework widgets. Loaded at lowest priority
/// so user stylesheets always win. This replaces the per-widget default_css()
/// static method which was never actually collected by the framework.
const BUILTIN_CSS: &str = r#"
Button { border: heavy; min-width: 16; height: 3; text-align: center; }
Checkbox { height: 1; }
Collapsible { min-height: 1; }
DataTable { border: rounded; min-height: 5; }
DirectoryTree { border: rounded; min-height: 5; flex-grow: 1; }
Footer { height: 1; }
Header { height: 1; }
Horizontal { layout-direction: horizontal; }
Input { border: rounded; height: 3; }
Label { min-height: 1; }
ListView { min-height: 3; flex-grow: 1; }
Log { min-height: 3; flex-grow: 1; }
Markdown { min-height: 3; }
MaskedInput { border: rounded; height: 3; }
Placeholder { border: rounded; min-height: 3; min-width: 10; }
ProgressBar { height: 1; }
RadioButton { height: 1; }
RadioSet { layout-direction: vertical; }
ScrollView { overflow: auto; }
Select { border: rounded; height: 3; }
Sparkline { height: 1; }
Switch { height: 1; }
TabbedContent { min-height: 3; layout-direction: vertical; }
TabBar { height: 1; }
TextArea { border: rounded; min-height: 5; }
Tree { border: rounded; min-height: 5; }
Vertical { layout-direction: vertical; }
"#;
use crate::css::render_style::paint_chrome;
use crate::event::dispatch::dispatch_message;
use crate::event::AppEvent;
use crate::layout::bridge::TaffyBridge;
use crate::layout::hit_map::MouseHitMap;
use crate::terminal::{init_panic_hook, TerminalGuard};
use crate::widget::context::AppContext;
use crate::widget::tree::{
    advance_focus, advance_focus_backward, clear_dirty_subtree, pop_screen, push_screen,
};
use crate::widget::{Widget, WidgetId};

/// Root application entry point for a textual-rs TUI application.
///
/// `App` owns the widget arena, layout engine, stylesheets, and event loop.
/// Create with [`App::new`], optionally add CSS with [`App::with_css`],
/// then call [`App::run`] to start the terminal UI.
///
/// # Example
///
/// ```no_run
/// # use textual_rs::{App, Widget};
/// # use textual_rs::widget::context::AppContext;
/// # use ratatui::{buffer::Buffer, layout::Rect};
/// # struct MyScreen;
/// # impl Widget for MyScreen {
/// #     fn render(&self, _: &AppContext, _: Rect, _: &mut Buffer) {}
/// #     fn widget_type_name(&self) -> &'static str { "MyScreen" }
/// # }
/// let mut app = App::new(|| Box::new(MyScreen))
///     .with_css("MyScreen { background: #0a0a0f; }");
/// // app.run().unwrap();
/// ```
pub struct App {
    ctx: AppContext,
    bridge: TaffyBridge,
    stylesheets: Vec<Stylesheet>,
    hit_map: Option<MouseHitMap>,
    root_screen_factory: Option<Box<dyn FnOnce() -> Box<dyn Widget>>>,
    /// Reactive graph owner — keeps signals and effects alive.
    /// Must be `Some` while the app is running. Stored as Option because
    /// it is initialized in run_async(), not in new().
    _owner: Option<Owner>,
    /// Receiver end of the dedicated worker result channel.
    /// The sender is stored on AppContext so widgets can spawn workers.
    worker_rx: Option<flume::Receiver<(WidgetId, Box<dyn std::any::Any + Send>)>>,
    /// Registry for app-level commands. Discover all commands via discover_all().
    command_registry: crate::command::CommandRegistry,
    /// Set after recomposition — forces full bridge sync on next render pass.
    needs_full_sync: bool,
    /// Timestamp of last Ctrl+C press for double-tap quit detection.
    last_ctrl_c: Option<std::time::Instant>,
    /// Current theme index in the builtin_themes list (for theme cycling).
    theme_index: usize,
    /// CSS files being watched for hot-reload. Polled every 2 seconds.
    watched_css: Vec<WatchedCss>,
    /// Tracks the last mouse-capture state sent to the terminal.
    /// Initialized to `true` because TerminalGuard enables mouse capture on creation.
    mouse_capture_active: bool,
    /// Key binding for cycling themes. Default: F5.
    /// Override with [`App::with_theme_cycle_key`] if needed.
    theme_cycle_key: (KeyCode, KeyModifiers),
    /// Optional callback invoked whenever the active theme changes.
    /// Receives the new theme name (e.g. `"tokyo-night"`).
    /// Use this to persist the user's theme choice across sessions.
    theme_changed_cb: Option<Box<dyn Fn(&str)>>,
}

/// A CSS file being watched for changes (hot-reload support).
struct WatchedCss {
    path: std::path::PathBuf,
    last_modified: std::time::SystemTime,
    /// Index into App::stylesheets where this file's parsed rules live.
    stylesheet_index: usize,
}

impl App {
    /// Create a new App instance with a screen factory closure.
    ///
    /// The factory is called once during `run()` to create the root screen widget.
    ///
    /// # Example
    /// ```no_run
    /// # use textual_rs::{App, Widget, WidgetId};
    /// # use textual_rs::widget::context::AppContext;
    /// # use ratatui::{buffer::Buffer, layout::Rect};
    /// # struct MyScreen;
    /// # impl Widget for MyScreen {
    /// #     fn render(&self, _: &AppContext, _: Rect, _: &mut Buffer) {}
    /// #     fn widget_type_name(&self) -> &'static str { "MyScreen" }
    /// # }
    /// let mut app = App::new(|| Box::new(MyScreen));
    /// ```
    pub fn new<F>(screen_factory: F) -> Self
    where
        F: FnOnce() -> Box<dyn Widget> + 'static,
    {
        // Parse built-in default CSS at lowest priority
        let (builtin_sheet, _) = Stylesheet::parse(BUILTIN_CSS);
        let mut ctx = AppContext::new();
        ctx.stylesheets.push(builtin_sheet.clone());

        App {
            ctx,
            bridge: TaffyBridge::new(),
            stylesheets: vec![builtin_sheet],
            hit_map: None,
            root_screen_factory: Some(Box::new(screen_factory)),
            _owner: None,
            worker_rx: None,
            command_registry: crate::command::CommandRegistry::new(),
            needs_full_sync: false,
            last_ctrl_c: None,
            theme_index: 0,
            watched_css: Vec::new(),
            mouse_capture_active: true,
            theme_cycle_key: (KeyCode::F(5), KeyModifiers::NONE),
            theme_changed_cb: None,
        }
    }

    /// Register an app-level command in the command palette.
    /// Registered commands appear in the palette alongside widget key bindings.
    pub fn register_command(&mut self, name: &str, action: &str) {
        self.command_registry.register(name, action);
    }

    /// Create an App without built-in CSS. Used by TestApp so widget tests get raw rendering
    /// without framework-default borders/sizes interfering with assertions.
    pub fn new_bare<F>(screen_factory: F) -> Self
    where
        F: FnOnce() -> Box<dyn Widget> + 'static,
    {
        App {
            ctx: AppContext::new(),
            bridge: TaffyBridge::new(),
            stylesheets: Vec::new(),
            hit_map: None,
            root_screen_factory: Some(Box::new(screen_factory)),
            _owner: None,
            worker_rx: None,
            command_registry: crate::command::CommandRegistry::new(),
            needs_full_sync: false,
            last_ctrl_c: None,
            theme_index: 0,
            watched_css: Vec::new(),
            mouse_capture_active: true,
            theme_cycle_key: (KeyCode::F(5), KeyModifiers::NONE),
            theme_changed_cb: None,
        }
    }

    /// Builder: parse and add a TCSS stylesheet. Parse errors are logged to stderr.
    pub fn with_css(mut self, css: &str) -> Self {
        let (stylesheet, errors) = Stylesheet::parse(css);
        for err in &errors {
            eprintln!("[textual-rs] CSS parse error: {}", err);
        }
        self.stylesheets.push(stylesheet.clone());
        self.ctx.stylesheets.push(stylesheet);
        self
    }

    /// Builder: load a TCSS file, parse it, and watch for changes (hot-reload).
    ///
    /// The file is read and parsed immediately. During `run()`, the file is polled
    /// every 2 seconds for changes. If modified, the CSS is re-parsed and styles
    /// are re-cascaded to all widgets.
    ///
    /// # Example
    /// ```no_run
    /// # use textual_rs::App;
    /// # struct S;
    /// # impl textual_rs::Widget for S {
    /// #     fn widget_type_name(&self) -> &'static str { "S" }
    /// #     fn render(&self, _: &textual_rs::widget::context::AppContext, _: ratatui::layout::Rect, _: &mut ratatui::buffer::Buffer) {}
    /// # }
    /// let app = App::new(|| Box::new(S))
    ///     .with_css_file("styles/app.tcss");
    /// ```
    pub fn with_css_file(mut self, path: impl AsRef<std::path::Path>) -> Self {
        let path = path.as_ref().to_path_buf();
        match std::fs::read_to_string(&path) {
            Ok(css_text) => {
                let (stylesheet, errors) = Stylesheet::parse(&css_text);
                for err in &errors {
                    eprintln!("[textual-rs] CSS parse error in {:?}: {}", path, err);
                }
                let index = self.stylesheets.len();
                self.stylesheets.push(stylesheet.clone());
                self.ctx.stylesheets.push(stylesheet);

                // Record for hot-reload polling
                let last_modified = std::fs::metadata(&path)
                    .and_then(|m| m.modified())
                    .unwrap_or(std::time::SystemTime::UNIX_EPOCH);
                self.watched_css.push(WatchedCss {
                    path,
                    last_modified,
                    stylesheet_index: index,
                });
            }
            Err(e) => {
                eprintln!("[textual-rs] Failed to read CSS file {:?}: {}", path, e);
            }
        }
        self
    }

    /// Set the active theme, triggering a full re-cascade of all widget styles.
    pub fn set_theme(&mut self, theme: crate::css::theme::Theme) {
        self.ctx.set_theme(theme);
        self.needs_full_sync = true;
    }

    /// Override the key binding used to cycle through built-in themes.
    ///
    /// The default is **F5** (`KeyCode::F(5)`, no modifiers). The previous default
    /// of Ctrl+T was changed because it is intercepted at the OS/terminal level on
    /// two common platforms before the application ever sees the keypress:
    /// - **macOS**: Ctrl+T is the `stty status` character — the terminal driver
    ///   sends `SIGINFO` to the process instead of forwarding the key.
    /// - **Windows Terminal**: Ctrl+T opens a new tab.
    ///
    /// ```no_run
    /// # use textual_rs::App;
    /// # use crossterm::event::{KeyCode, KeyModifiers};
    /// # struct MyScreen;
    /// # impl textual_rs::Widget for MyScreen {
    /// #     fn render(&self, _: &textual_rs::widget::context::AppContext, _: ratatui::layout::Rect, _: &mut ratatui::buffer::Buffer) {}
    /// #     fn widget_type_name(&self) -> &'static str { "MyScreen" }
    /// # }
    /// let app = App::new(|| Box::new(MyScreen))
    ///     .with_theme_cycle_key(KeyCode::Char('t'), crossterm::event::KeyModifiers::CONTROL);
    /// ```
    pub fn with_theme_cycle_key(mut self, code: KeyCode, modifiers: KeyModifiers) -> Self {
        self.theme_cycle_key = (code, modifiers);
        self
    }

    /// Register a callback invoked whenever the active theme changes.
    ///
    /// The callback receives the new theme name (e.g. `"tokyo-night"`).
    /// Use this to persist the user's theme choice across sessions.
    ///
    /// ```no_run
    /// # use textual_rs::App;
    /// # struct MyScreen;
    /// # impl textual_rs::Widget for MyScreen {
    /// #     fn render(&self, _: &textual_rs::widget::context::AppContext, _: ratatui::layout::Rect, _: &mut ratatui::buffer::Buffer) {}
    /// #     fn widget_type_name(&self) -> &'static str { "MyScreen" }
    /// # }
    /// let app = App::new(|| Box::new(MyScreen))
    ///     .on_theme_changed(|name| {
    ///         // e.g. write name to a config file
    ///         eprintln!("theme changed to: {name}");
    ///     });
    /// ```
    pub fn on_theme_changed<F: Fn(&str) + 'static>(mut self, f: F) -> Self {
        self.theme_changed_cb = Some(Box::new(f));
        self
    }

    /// Cycle to the next built-in theme (wraps around). Used by the theme cycle key.
    fn cycle_theme(&mut self) {
        let themes = crate::css::theme::builtin_themes();
        self.theme_index = (self.theme_index + 1) % themes.len();
        let theme = themes.into_iter().nth(self.theme_index).unwrap();
        let name = theme.name.clone();
        self.set_theme(theme);
        if let Some(ref cb) = self.theme_changed_cb {
            cb(&name);
        }
    }

    /// Run the application. Blocks the calling thread until the user quits.
    /// Creates its own single-threaded Tokio runtime internally.
    pub fn run(&mut self) -> Result<()> {
        init_panic_hook();
        let rt = Builder::new_current_thread().enable_all().build()?;
        let local = LocalSet::new();
        local.block_on(&rt, self.run_async())
    }

    async fn run_async(&mut self) -> Result<()> {
        // Initialize reactive_graph's executor (uses tokio::task::spawn_local under the hood).
        // Safe to call multiple times — returns Err on subsequent calls which we ignore.
        let _ = any_spawner::Executor::init_tokio();
        // Create reactive Owner scope — all signals/effects created during app lifetime
        // are tied to this owner. Dropping it cancels all effects.
        self._owner = Some(Owner::new());

        let _guard = TerminalGuard::new()?;
        let backend = CrosstermBackend::new(std::io::stdout());
        let mut terminal = Terminal::new(backend)?;

        #[cfg(debug_assertions)]
        eprintln!("[textual-rs] Terminal: {:?}", self.ctx.terminal_caps);

        // Mount root screen
        if let Some(factory) = self.root_screen_factory.take() {
            let root_screen = factory();
            push_screen(root_screen, &mut self.ctx);
        }

        // Initial render
        self.full_render_pass(&mut terminal)?;

        let (tx, rx) = flume::unbounded::<AppEvent>();

        // Store event sender on AppContext so widgets and effects can post events.
        self.ctx.event_tx = Some(tx.clone());

        // Create dedicated worker result channel. The sender is stored on AppContext
        // so run_worker can send results; we poll the receiver in the select! loop.
        let (worker_tx, worker_rx) =
            flume::unbounded::<(WidgetId, Box<dyn std::any::Any + Send>)>();
        self.ctx.worker_tx = Some(worker_tx);
        self.worker_rx = Some(worker_rx);

        // Spawn EventStream reader task on LocalSet (does not need Send)
        task::spawn_local(async move {
            let mut stream = EventStream::new();
            while let Some(Ok(event)) = stream.next().await {
                let app_event = match event {
                    Event::Key(k) => Some(AppEvent::Key(k)),
                    Event::Mouse(m) => Some(AppEvent::Mouse(m)),
                    Event::Resize(c, r) => Some(AppEvent::Resize(c, r)),
                    _ => None,
                };
                if let Some(e) = app_event {
                    if tx.send(e).is_err() {
                        break;
                    }
                }
            }
        });

        // Take worker_rx out of self to avoid borrow issues in select!
        let worker_rx = self.worker_rx.take().expect("worker_rx not initialized");

        // CSS hot-reload polling interval (only active if watching files)
        let mut css_poll = tokio::time::interval(std::time::Duration::from_secs(2));
        css_poll.tick().await; // consume the immediate first tick

        // Main event loop — select! between app events, worker results, and CSS poll
        loop {
            tokio::select! {
                // Render tick — drives animations and CSS hot-reload checks.
                // 30fps is sufficient for smooth tweens while keeping CPU usage low.
                // ratatui's diff engine means unchanged frames have near-zero terminal I/O.
                _ = tokio::time::sleep(std::time::Duration::from_millis(33)) => {
                    self.full_render_pass(&mut terminal)?;
                }

                event = rx.recv_async() => {
                    match event {
                        // Ignore non-press key events (release, repeat on some platforms)
                        Ok(AppEvent::Key(k)) if k.kind != KeyEventKind::Press => {}

                        Ok(AppEvent::Key(k)) => {
                            // 0a. If overlay is active, route key event to it
                            if self.ctx.active_overlay.borrow().is_some() {
                                {
                                    let overlay = self.ctx.active_overlay.borrow();
                                    if let Some(ref widget) = *overlay {
                                        // First try key bindings
                                        let mut binding_handled = false;
                                        for binding in widget.key_bindings() {
                                            if binding.matches(k.code, k.modifiers) {
                                                widget.on_action(binding.action, &self.ctx);
                                                binding_handled = true;
                                                break;
                                            }
                                        }
                                        if !binding_handled {
                                            // Then try on_event (for char input, etc.)
                                            widget.on_event(&k as &dyn std::any::Any, &self.ctx);
                                        }
                                    }
                                }
                                // Drain deferred dismiss
                                if self.ctx.pending_overlay_dismiss.get() {
                                    self.ctx.pending_overlay_dismiss.set(false);
                                    *self.ctx.active_overlay.borrow_mut() = None;
                                }
                                self.full_render_pass(&mut terminal)?;
                                continue;
                            }

                            // 0. Ctrl+P: open command palette
                            if k.code == KeyCode::Char('p')
                                && k.modifiers.contains(KeyModifiers::CONTROL)
                            {
                                let commands = self.command_registry.discover_all(&self.ctx);
                                let palette = crate::command::CommandPalette::new(commands);
                                *self.ctx.active_overlay.borrow_mut() = Some(Box::new(palette));
                                self.full_render_pass(&mut terminal)?;
                                continue;
                            }

                            // 0b. theme cycle key (default F5, configurable via with_theme_cycle_key)
                            if k.code == self.theme_cycle_key.0
                                && k.modifiers.contains(self.theme_cycle_key.1)
                            {
                                self.cycle_theme();
                                self.full_render_pass(&mut terminal)?;
                                continue;
                            }

                            // Ctrl+C: route to copy if focused widget has text selection,
                            // otherwise use double-tap-to-quit pattern.
                            if k.code == KeyCode::Char('c')
                                && k.modifiers.contains(KeyModifiers::CONTROL)
                            {
                                let widget_has_selection = self.ctx.focused_widget
                                    .and_then(|id| self.ctx.arena.get(id))
                                    .map(|w| w.has_text_selection())
                                    .unwrap_or(false);

                                if widget_has_selection {
                                    // Route to widget's copy action via key bindings
                                    if let Some(focused_id) = self.ctx.focused_widget {
                                        if let Some(widget) = self.ctx.arena.get(focused_id) {
                                            for binding in widget.key_bindings() {
                                                if binding.matches(k.code, k.modifiers) {
                                                    widget.on_action(binding.action, &self.ctx);
                                                    break;
                                                }
                                            }
                                        }
                                    }
                                    self.last_ctrl_c = None;
                                    self.full_render_pass(&mut terminal)?;
                                    continue;
                                }

                                // No selection: double-tap to quit (within 500ms)
                                let now = std::time::Instant::now();
                                if let Some(last) = self.last_ctrl_c {
                                    if now.duration_since(last).as_millis() < 500 {
                                        break; // Double-tap -- quit
                                    }
                                }
                                self.last_ctrl_c = Some(now);
                                self.full_render_pass(&mut terminal)?;
                                continue;
                            }

                            // 2. Check focused widget's key bindings
                            let mut handled = false;
                            if let Some(focused_id) = self.ctx.focused_widget {
                                if let Some(widget) = self.ctx.arena.get(focused_id) {
                                    for binding in widget.key_bindings() {
                                        if binding.matches(k.code, k.modifiers) {
                                            widget.on_action(binding.action, &self.ctx);
                                            handled = true;
                                            break;
                                        }
                                    }
                                }
                            }

                            // 2b. Check active screen's key bindings (screens can't be focused
                            //     directly, so their bindings are never reached via step 2).
                            if !handled {
                                if let Some(&screen_id) = self.ctx.screen_stack.last() {
                                    if let Some(screen) = self.ctx.arena.get(screen_id) {
                                        for binding in screen.key_bindings() {
                                            if binding.matches(k.code, k.modifiers) {
                                                screen.on_action(binding.action, &self.ctx);
                                                handled = true;
                                                break;
                                            }
                                        }
                                    }
                                }
                            }

                            // 3. If not handled by binding, dispatch as key event to focused widget, then bubble.
                            //    If no widget is focused, fall back to dispatching to the screen root.
                            if !handled {
                                if let Some(focused_id) = self.ctx.focused_widget {
                                    dispatch_message(focused_id, &k, &self.ctx);
                                    handled = true;
                                } else if let Some(&screen_id) = self.ctx.screen_stack.last() {
                                    dispatch_message(screen_id, &k, &self.ctx);
                                    handled = true;
                                }
                            }

                            // 3.5. Shift+F10 or Menu key → open context menu
                            if !handled {
                                let is_context_key = matches!(k.code, KeyCode::F(10) if k.modifiers.contains(KeyModifiers::SHIFT))
                                    || matches!(k.code, KeyCode::Menu);
                                if is_context_key {
                                    if let Some(focused_id) = self.ctx.focused_widget {
                                        if let Some(widget) = self.ctx.arena.get(focused_id) {
                                            let items = widget.context_menu_items();
                                            if !items.is_empty() {
                                                // Position at widget's top-left (approximate)
                                                let (ax, ay) = self.hit_map.as_ref()
                                                    .and({
                                                        // Use first cell of focused widget as anchor
                                                        None::<(u16, u16)> // fallback below
                                                    })
                                                    .unwrap_or((0, 0));
                                                let overlay = crate::widget::context_menu::ContextMenuOverlay::new(
                                                    items, Some(focused_id), ax, ay,
                                                );
                                                *self.ctx.active_overlay.borrow_mut() = Some(Box::new(overlay));
                                                handled = true;
                                            }
                                        }
                                    }
                                }
                            }

                            // 4. App-level key handling (Tab for focus cycling)
                            if !handled || matches!(k.code, KeyCode::Tab) {
                                match k.code {
                                    KeyCode::Tab if k.modifiers.contains(KeyModifiers::SHIFT) => {
                                        advance_focus_backward(&mut self.ctx);
                                    }
                                    KeyCode::Tab => {
                                        advance_focus(&mut self.ctx);
                                    }
                                    _ => {}
                                }
                            }

                            // 5. Drain message queue + process deferred screens
                            self.drain_message_queue();
                            self.drain_mouse_capture_changes();
                            self.process_deferred_screens();
                            self.full_render_pass(&mut terminal)?;
                        }

                        Ok(AppEvent::Mouse(m)) => {
                            use crossterm::event::{MouseEventKind, MouseButton};

                            // Shift+mouse bypasses capture -- let terminal handle native text selection.
                            // When Shift is held, crossterm still delivers the event but we ignore it,
                            // allowing the terminal emulator to handle selection natively.
                            if m.modifiers.contains(KeyModifiers::SHIFT) {
                                continue;
                            }

                            // If mouse capture is disabled (stack top = false), skip all mouse handling.
                            // Events may still arrive briefly after DisableMouseCapture is sent.
                            if !self.ctx.mouse_capture_stack.is_enabled() {
                                continue;
                            }

                            // Route to overlay if active
                            if self.ctx.active_overlay.borrow().is_some() {
                                {
                                    let overlay = self.ctx.active_overlay.borrow();
                                    if let Some(ref widget) = *overlay {
                                        widget.on_event(&m as &dyn std::any::Any, &self.ctx);
                                    }
                                }
                                if self.ctx.pending_overlay_dismiss.get() {
                                    self.ctx.pending_overlay_dismiss.set(false);
                                    *self.ctx.active_overlay.borrow_mut() = None;
                                }
                                self.full_render_pass(&mut terminal)?;
                                continue;
                            }

                            match m.kind {
                                MouseEventKind::Down(MouseButton::Right) => {
                                    // Right-click: walk parent chain to find context menu items
                                    if let Some(ref hit_map) = self.hit_map {
                                        if let Some(target_id) = hit_map.hit_test(m.column, m.row) {
                                            let chain = crate::event::dispatch::collect_parent_chain(target_id, &self.ctx);
                                            for &id in &chain {
                                                if let Some(widget) = self.ctx.arena.get(id) {
                                                    let items = widget.context_menu_items();
                                                    if !items.is_empty() {
                                                        let overlay = crate::widget::context_menu::ContextMenuOverlay::new(
                                                            items, Some(id), m.column, m.row,
                                                        );
                                                        *self.ctx.active_overlay.borrow_mut() = Some(Box::new(overlay));
                                                        self.process_deferred_screens();
                                                        self.full_render_pass(&mut terminal)?;
                                                        break;
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                                MouseEventKind::Down(_) => {
                                    // Left/middle click: focus + activate
                                    if let Some(ref hit_map) = self.hit_map {
                                        if let Some(target_id) = hit_map.hit_test(m.column, m.row) {
                                            if let Some(widget) = self.ctx.arena.get(target_id) {
                                                if widget.can_focus() {
                                                    self.ctx.focused_widget = Some(target_id);
                                                }
                                                if let Some(action) = widget.click_action() {
                                                    widget.on_action(action, &self.ctx);
                                                }
                                            }
                                            dispatch_message(target_id, &m, &self.ctx);
                                            self.drain_message_queue();
                                            self.drain_mouse_capture_changes();
                                            self.process_deferred_screens();
                                            self.full_render_pass(&mut terminal)?;
                                        }
                                    }
                                }
                                MouseEventKind::ScrollDown
                                | MouseEventKind::ScrollUp
                                | MouseEventKind::ScrollLeft
                                | MouseEventKind::ScrollRight => {
                                    let action = match m.kind {
                                        MouseEventKind::ScrollUp => "scroll_up",
                                        MouseEventKind::ScrollDown => "scroll_down",
                                        MouseEventKind::ScrollLeft => "scroll_left",
                                        MouseEventKind::ScrollRight => "scroll_right",
                                        _ => unreachable!(),
                                    };
                                    if let Some(ref hit_map) = self.hit_map {
                                        if let Some(target_id) = hit_map.hit_test(m.column, m.row) {
                                            // Dispatch as action, bubbling up the parent chain
                                            // so scrollable containers (Log, ScrollView, ListView)
                                            // respond to mouse wheel.
                                            dispatch_scroll_action(target_id, action, &self.ctx);
                                            self.drain_message_queue();
                                            self.drain_mouse_capture_changes();
                                            self.process_deferred_screens();
                                            self.full_render_pass(&mut terminal)?;
                                        }
                                    }
                                }
                                MouseEventKind::Moved => {
                                    if let Some(ref hit_map) = self.hit_map {
                                        let new_hover = hit_map.hit_test(m.column, m.row);
                                        let old_hover = self.ctx.hovered_widget;
                                        if new_hover != old_hover {
                                            // Clear old hover pseudo-class
                                            if let Some(old_id) = old_hover {
                                                if let Some(pcs) = self.ctx.pseudo_classes.get_mut(old_id) {
                                                    pcs.remove(&crate::css::types::PseudoClass::Hover);
                                                }
                                            }
                                            // Set new hover pseudo-class
                                            if let Some(new_id) = new_hover {
                                                if let Some(pcs) = self.ctx.pseudo_classes.get_mut(new_id) {
                                                    pcs.insert(crate::css::types::PseudoClass::Hover);
                                                }
                                            }
                                            self.ctx.hovered_widget = new_hover;
                                            self.full_render_pass(&mut terminal)?;
                                        }
                                    }
                                }
                                _ => {} // Ignore drag for now
                            }
                        }

                        Ok(AppEvent::Resize(_, _)) => {
                            self.needs_full_sync = true;
                            // Re-sync mouse capture state after resize to prevent
                            // terminal state from desynchronizing.
                            {
                                use crossterm::event::{DisableMouseCapture, EnableMouseCapture};
                                use crossterm::execute;
                                if self.ctx.mouse_capture_stack.is_enabled() {
                                    let _ = execute!(std::io::stdout(), EnableMouseCapture);
                                } else {
                                    let _ = execute!(std::io::stdout(), DisableMouseCapture);
                                }
                                self.mouse_capture_active = self.ctx.mouse_capture_stack.is_enabled();
                            }
                            self.full_render_pass(&mut terminal)?;
                        }
                        Ok(AppEvent::RenderRequest) => {
                            // Coalesce: drain any additional RenderRequests queued in the same tick
                            while let Ok(AppEvent::RenderRequest) = rx.try_recv() {}
                            self.full_render_pass(&mut terminal)?;
                        }
                        Ok(_) => {}
                        Err(_) => break, // channel closed
                    }
                }

                result = worker_rx.recv_async() => {
                    if let Ok((source_id, payload)) = result {
                        self.ctx.message_queue.borrow_mut().push((source_id, payload));
                        self.drain_message_queue();
                        self.drain_mouse_capture_changes();
                        self.process_deferred_screens();
                        self.full_render_pass(&mut terminal)?;
                    }
                }

                _ = css_poll.tick(), if !self.watched_css.is_empty() => {
                    // Check watched CSS files for changes
                    let mut any_changed = false;
                    for watch in &mut self.watched_css {
                        if let Ok(meta) = std::fs::metadata(&watch.path) {
                            if let Ok(modified) = meta.modified() {
                                if modified > watch.last_modified {
                                    watch.last_modified = modified;
                                    if let Ok(css_text) = std::fs::read_to_string(&watch.path) {
                                        let (new_sheet, errors) = Stylesheet::parse(&css_text);
                                        for err in &errors {
                                            eprintln!("[textual-rs] CSS hot-reload parse error in {:?}: {}", watch.path, err);
                                        }
                                        // Replace the stylesheet at the stored index
                                        if watch.stylesheet_index < self.stylesheets.len() {
                                            self.stylesheets[watch.stylesheet_index] = new_sheet.clone();
                                        }
                                        if watch.stylesheet_index < self.ctx.stylesheets.len() {
                                            self.ctx.stylesheets[watch.stylesheet_index] = new_sheet;
                                        }
                                        any_changed = true;
                                        #[cfg(debug_assertions)]
                                        eprintln!("[textual-rs] Hot-reloaded CSS from {:?}", watch.path);
                                    }
                                }
                            }
                        }
                    }
                    if any_changed {
                        self.needs_full_sync = true;
                        self.full_render_pass(&mut terminal)?;
                    }
                }
            }
        }

        Ok(())
    }

    /// Core render integration: cascade → layout → clear dirty → hit map → draw.
    fn full_render_pass<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> Result<()>
    where
        B::Error: Send + Sync + 'static,
    {
        let screen_id = match self.ctx.screen_stack.last().copied() {
            Some(id) => id,
            None => return Ok(()),
        };

        // a. Apply CSS cascade (top screen only — background screens are frozen)
        apply_cascade_to_tree(screen_id, &self.stylesheets, &mut self.ctx);

        // b. Sync layout tree (top screen only)
        if self.needs_full_sync {
            self.bridge.sync_subtree(screen_id, &self.ctx);
            self.needs_full_sync = false;
        } else {
            self.bridge.sync_dirty_subtree(screen_id, &self.ctx);
        }

        // c. Compute layout (top screen only; background screen layouts are preserved in cache)
        let size = terminal.size()?;
        self.bridge
            .compute_layout(screen_id, size.width, size.height, &self.ctx);

        // d. Clear dirty flags (top screen only)
        clear_dirty_subtree(screen_id, &mut self.ctx);

        // e. Build hit map from top screen only (ensures mouse is blocked from background screens)
        let dfs_ids = collect_subtree_dfs(screen_id, &self.ctx);
        self.hit_map = Some(MouseHitMap::build(&dfs_ids, self.bridge.layout_cache()));

        // f. Render all screens bottom-to-top for correct modal layering.
        //    Background screens are frozen (no re-layout) but still rendered so modals
        //    appear visually overlaid on the content behind them.
        let all_screens: Vec<WidgetId> = self.ctx.screen_stack.clone();
        let ctx_ref = &self.ctx;
        let bridge_ref = &self.bridge;
        terminal.draw(|frame| {
            for &sid in &all_screens {
                render_widget_tree(sid, ctx_ref, bridge_ref, frame);
            }
        })?;

        // Advance spinner tick for next frame (all loading overlays animate in sync)
        self.ctx.spinner_tick.set(self.ctx.spinner_tick.get().wrapping_add(1));

        // Advance toast countdowns and remove expired toasts
        {
            let mut toasts = self.ctx.toast_entries.borrow_mut();
            crate::widget::toast::tick_toasts(&mut toasts);
        }

        Ok(())
    }

    /// Render one frame to a TestBackend and return the resulting buffer.
    /// This is the Phase 2 test entry point for layout geometry assertions.
    pub fn render_to_test_backend(&mut self, cols: u16, rows: u16) -> ratatui::buffer::Buffer {
        let backend = TestBackend::new(cols, rows);
        let mut terminal = Terminal::new(backend).expect("failed to create TestBackend terminal");

        // Mount root screen if not yet mounted
        if self.ctx.screen_stack.is_empty() {
            if let Some(factory) = self.root_screen_factory.take() {
                let root_screen = factory();
                push_screen(root_screen, &mut self.ctx);
            }
        }

        let screen_id = match self.ctx.screen_stack.last().copied() {
            Some(id) => id,
            None => return ratatui::buffer::Buffer::empty(ratatui::layout::Rect::default()),
        };

        // Cascade + layout
        apply_cascade_to_tree(screen_id, &self.stylesheets, &mut self.ctx);
        self.bridge.sync_dirty_subtree(screen_id, &self.ctx);
        self.bridge.compute_layout(screen_id, cols, rows, &self.ctx);
        clear_dirty_subtree(screen_id, &mut self.ctx);

        // Build hit map
        let dfs_ids = collect_subtree_dfs(screen_id, &self.ctx);
        self.hit_map = Some(MouseHitMap::build(&dfs_ids, self.bridge.layout_cache()));

        // Draw
        let ctx_ref = &self.ctx;
        let bridge_ref = &self.bridge;
        terminal
            .draw(|frame| {
                render_widget_tree(screen_id, ctx_ref, bridge_ref, frame);
            })
            .expect("failed to draw to TestBackend");

        // Advance spinner tick for next frame (all loading overlays animate in sync)
        self.ctx.spinner_tick.set(self.ctx.spinner_tick.get().wrapping_add(1));

        terminal.backend().buffer().clone()
    }

    /// Expose AppContext for test assertions (e.g., finding widget IDs, inspecting focus).
    pub fn ctx(&self) -> &AppContext {
        &self.ctx
    }

    /// Expose TaffyBridge for test assertions (e.g., verifying computed Rects).
    pub fn bridge(&self) -> &TaffyBridge {
        &self.bridge
    }

    /// Set the event sender on AppContext. Used by TestApp for headless testing.
    pub fn set_event_tx(&mut self, tx: flume::Sender<AppEvent>) {
        self.ctx.event_tx = Some(tx);
    }

    /// Enable or disable animation snapping. When true, animations snap to their
    /// target value instead of interpolating. Used by TestApp for deterministic rendering.
    pub fn set_skip_animations(&mut self, skip: bool) {
        self.ctx.skip_animations = skip;
    }

    /// Mount the root screen. Calls the stored factory and pushes the screen onto the stack.
    /// No-op if the factory was already consumed (screen already mounted).
    pub fn mount_root_screen(&mut self) {
        if let Some(factory) = self.root_screen_factory.take() {
            let root_screen = factory();
            push_screen(root_screen, &mut self.ctx);
        }
    }

    /// Render one frame to the provided terminal. Public for TestApp.
    pub fn render_to_terminal<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> Result<()>
    where
        B::Error: Send + Sync + 'static,
    {
        self.full_render_pass(terminal)
    }

    /// Handle a key event: check bindings, dispatch to focused widget, advance focus on Tab.
    /// Returns true if the event was handled by a binding or on_event handler.
    pub fn handle_key_event(&mut self, k: crossterm::event::KeyEvent) -> bool {
        // 0a. If overlay is active, route keys to it
        if self.ctx.active_overlay.borrow().is_some() {
            {
                let overlay = self.ctx.active_overlay.borrow();
                if let Some(ref widget) = *overlay {
                    // First try key bindings
                    let mut binding_handled = false;
                    for binding in widget.key_bindings() {
                        if binding.matches(k.code, k.modifiers) {
                            widget.on_action(binding.action, &self.ctx);
                            binding_handled = true;
                            break;
                        }
                    }
                    if !binding_handled {
                        // Then try on_event (for char input, etc.)
                        widget.on_event(&k as &dyn std::any::Any, &self.ctx);
                    }
                }
            }
            // Drain deferred dismiss
            if self.ctx.pending_overlay_dismiss.get() {
                self.ctx.pending_overlay_dismiss.set(false);
                *self.ctx.active_overlay.borrow_mut() = None;
            }
            return true;
        }

        // 0. Ctrl+P: open command palette
        if k.code == KeyCode::Char('p') && k.modifiers.contains(KeyModifiers::CONTROL) {
            let commands = self.command_registry.discover_all(&self.ctx);
            let palette = crate::command::CommandPalette::new(commands);
            *self.ctx.active_overlay.borrow_mut() = Some(Box::new(palette));
            return true;
        }

        // 0b. theme cycle key (default F5, configurable via with_theme_cycle_key)
        if k.code == self.theme_cycle_key.0 && k.modifiers.contains(self.theme_cycle_key.1) {
            self.cycle_theme();
            return true;
        }

        // 1. Check focused widget's key bindings
        let mut handled = false;
        if let Some(focused_id) = self.ctx.focused_widget {
            if let Some(widget) = self.ctx.arena.get(focused_id) {
                for binding in widget.key_bindings() {
                    if binding.matches(k.code, k.modifiers) {
                        widget.on_action(binding.action, &self.ctx);
                        handled = true;
                        break;
                    }
                }
            }
        }

        // 1b. Check active screen's key bindings (screens can't be focused directly,
        //     so their bindings are never reached via step 1 — check them explicitly).
        if !handled {
            if let Some(&screen_id) = self.ctx.screen_stack.last() {
                if let Some(screen) = self.ctx.arena.get(screen_id) {
                    for binding in screen.key_bindings() {
                        if binding.matches(k.code, k.modifiers) {
                            screen.on_action(binding.action, &self.ctx);
                            handled = true;
                            break;
                        }
                    }
                }
            }
        }

        // 2. If not handled by binding, dispatch as key event to focused widget, then bubble.
        //    If no widget is focused (e.g. screen has only non-focusable children), fall back
        //    to dispatching directly to the screen root so it still receives key events.
        if !handled {
            if let Some(focused_id) = self.ctx.focused_widget {
                dispatch_message(focused_id, &k, &self.ctx);
                handled = true;
            } else if let Some(&screen_id) = self.ctx.screen_stack.last() {
                dispatch_message(screen_id, &k, &self.ctx);
                handled = true;
            }
        }

        // 3. App-level key handling (Tab for focus cycling)
        if !handled || matches!(k.code, KeyCode::Tab) {
            match k.code {
                KeyCode::Tab if k.modifiers.contains(KeyModifiers::SHIFT) => {
                    advance_focus_backward(&mut self.ctx);
                }
                KeyCode::Tab => {
                    advance_focus(&mut self.ctx);
                }
                _ => {}
            }
        }

        handled
    }

    /// Handle a mouse event: hit-test and dispatch to target widget.
    pub fn handle_mouse_event(&mut self, m: crossterm::event::MouseEvent) {
        use crossterm::event::{MouseButton, MouseEventKind};

        // Shift+mouse bypasses capture -- native text selection passthrough.
        if m.modifiers.contains(KeyModifiers::SHIFT) {
            return;
        }

        // If mouse capture is disabled via stack, skip all mouse handling.
        if !self.ctx.mouse_capture_stack.is_enabled() {
            return;
        }

        // If an overlay is active, route events to it first
        if self.ctx.active_overlay.borrow().is_some() {
            let overlay = self.ctx.active_overlay.borrow();
            if let Some(ref widget) = *overlay {
                widget.on_event(&m as &dyn std::any::Any, &self.ctx);
            }
            drop(overlay);
            // Drain deferred overlay dismiss
            if self.ctx.pending_overlay_dismiss.get() {
                self.ctx.pending_overlay_dismiss.set(false);
                *self.ctx.active_overlay.borrow_mut() = None;
            }
            return;
        }
        match m.kind {
            MouseEventKind::Down(MouseButton::Right) => {
                // Right-click: spawn context menu. Walk parent chain to find items.
                if let Some(ref hit_map) = self.hit_map {
                    if let Some(target_id) = hit_map.hit_test(m.column, m.row) {
                        let chain =
                            crate::event::dispatch::collect_parent_chain(target_id, &self.ctx);
                        for &id in &chain {
                            if let Some(widget) = self.ctx.arena.get(id) {
                                let items = widget.context_menu_items();
                                if !items.is_empty() {
                                    let overlay =
                                        crate::widget::context_menu::ContextMenuOverlay::new(
                                            items,
                                            Some(id),
                                            m.column,
                                            m.row,
                                        );
                                    *self.ctx.active_overlay.borrow_mut() = Some(Box::new(overlay));
                                    break;
                                }
                            }
                        }
                    }
                }
            }
            MouseEventKind::Down(_) => {
                if let Some(ref hit_map) = self.hit_map {
                    if let Some(target_id) = hit_map.hit_test(m.column, m.row) {
                        if let Some(widget) = self.ctx.arena.get(target_id) {
                            if widget.can_focus() {
                                self.ctx.focused_widget = Some(target_id);
                            }
                            if let Some(action) = widget.click_action() {
                                widget.on_action(action, &self.ctx);
                            }
                        }
                        dispatch_message(target_id, &m, &self.ctx);
                    }
                }
            }
            MouseEventKind::ScrollDown
            | MouseEventKind::ScrollUp
            | MouseEventKind::ScrollLeft
            | MouseEventKind::ScrollRight => {
                let action = match m.kind {
                    MouseEventKind::ScrollUp => "scroll_up",
                    MouseEventKind::ScrollDown => "scroll_down",
                    MouseEventKind::ScrollLeft => "scroll_left",
                    MouseEventKind::ScrollRight => "scroll_right",
                    _ => unreachable!(),
                };
                if let Some(ref hit_map) = self.hit_map {
                    if let Some(target_id) = hit_map.hit_test(m.column, m.row) {
                        dispatch_scroll_action(target_id, action, &self.ctx);
                    }
                }
            }
            MouseEventKind::Moved => {
                if let Some(ref hit_map) = self.hit_map {
                    let new_hover = hit_map.hit_test(m.column, m.row);
                    let old_hover = self.ctx.hovered_widget;
                    if new_hover != old_hover {
                        if let Some(old_id) = old_hover {
                            if let Some(pcs) = self.ctx.pseudo_classes.get_mut(old_id) {
                                pcs.remove(&crate::css::types::PseudoClass::Hover);
                            }
                        }
                        if let Some(new_id) = new_hover {
                            if let Some(pcs) = self.ctx.pseudo_classes.get_mut(new_id) {
                                pcs.insert(crate::css::types::PseudoClass::Hover);
                            }
                        }
                        self.ctx.hovered_widget = new_hover;
                    }
                }
            }
            _ => {}
        }
    }

    /// Drain pending screen pushes and pops scheduled by widgets via
    /// push_screen_deferred() / pop_screen_deferred(). Called after each event cycle.
    pub fn process_deferred_screens(&mut self) {
        // Process pops first, then pushes (pop old modal before pushing new one).
        // Guard: never pop the last screen (pop_screen_deferred is a no-op on last screen).
        let pops = self.ctx.pending_screen_pops.get();
        if pops > 0 {
            self.ctx.pending_screen_pops.set(0);
            for _ in 0..pops {
                // Refuse to pop if only one screen remains
                if self.ctx.screen_stack.len() <= 1 {
                    break;
                }
                // Grab the top screen id before popping so we can match its result sender.
                let top_screen_id = self.ctx.screen_stack.last().copied();
                // Take the result value (set by pop_screen_with, if any).
                let result = self.ctx.pending_pop_result.borrow_mut().take();

                pop_screen(&mut self.ctx);

                // If there was a pending result and a registered sender for the popped screen,
                // deliver the result through the oneshot channel now.
                if let (Some(top_id), Some(value)) = (top_screen_id, result) {
                    if let Some(sender) = self.ctx.screen_result_senders.borrow_mut().remove(&top_id) {
                        let _ = sender.send(value);
                    }
                }
            }
            self.needs_full_sync = true;
        }

        let pushes: Vec<Box<dyn Widget>> = self
            .ctx
            .pending_screen_pushes
            .borrow_mut()
            .drain(..)
            .collect();
        for screen in pushes {
            push_screen(screen, &mut self.ctx);
        }

        // Process push_screen_wait requests: push the screen and record the sender keyed by the new screen id.
        let wait_pushes: Vec<(Box<dyn Widget>, tokio::sync::oneshot::Sender<Box<dyn std::any::Any + Send>>)> =
            self.ctx.pending_screen_wait_pushes.borrow_mut().drain(..).collect();
        for (screen, sender) in wait_pushes {
            let screen_id = push_screen(screen, &mut self.ctx);
            self.ctx.screen_result_senders.borrow_mut().insert(screen_id, sender);
            self.needs_full_sync = true;
        }

        // Process pending recompositions (e.g. tab switching)
        let recompose_ids: Vec<WidgetId> =
            self.ctx.pending_recompose.borrow_mut().drain(..).collect();
        if !recompose_ids.is_empty() {
            for id in recompose_ids {
                crate::widget::tree::recompose_widget(id, &mut self.ctx);
            }
            // Flag that next render pass needs a full sync (not dirty-only)
            self.needs_full_sync = true;
        }
    }

    /// Drain the message queue, dispatching each message through bubbling.
    /// Handles recursive message posting (widget handlers posting new messages)
    /// up to a depth of 100 iterations to prevent infinite loops.
    pub fn drain_message_queue(&self) {
        // Take all messages out of the RefCell (avoids borrow conflict during dispatch)
        let messages: Vec<_> = self.ctx.message_queue.borrow_mut().drain(..).collect();
        for (source, message) in messages {
            dispatch_message(source, message.as_ref(), &self.ctx);
        }
        // Check if dispatching produced new messages (recursive drain, bounded)
        for _ in 0..100 {
            let more: Vec<_> = self.ctx.message_queue.borrow_mut().drain(..).collect();
            if more.is_empty() {
                break;
            }
            for (source, message) in more {
                dispatch_message(source, message.as_ref(), &self.ctx);
            }
        }
    }

    /// Drain deferred mouse capture pushes/pops from widgets and sync terminal state.
    /// Called after drain_message_queue() in the event loop.
    fn drain_mouse_capture_changes(&mut self) {
        use crossterm::event::{DisableMouseCapture, EnableMouseCapture};
        use crossterm::execute;

        // Drain pending pushes
        let pushes: Vec<bool> = self.ctx.pending_mouse_push.borrow_mut().drain(..).collect();
        for enabled in pushes {
            self.ctx.mouse_capture_stack.push(enabled);
        }

        // Drain pending pops
        let pop_count = self.ctx.pending_mouse_pops.get();
        if pop_count > 0 {
            for _ in 0..pop_count {
                self.ctx.mouse_capture_stack.pop();
            }
            self.ctx.pending_mouse_pops.set(0);
        }

        // Sync terminal state if changed
        let desired = self.ctx.mouse_capture_stack.is_enabled();
        if desired != self.mouse_capture_active {
            if desired {
                let _ = execute!(std::io::stdout(), EnableMouseCapture);
            } else {
                let _ = execute!(std::io::stdout(), DisableMouseCapture);
            }
            self.mouse_capture_active = desired;
        }
    }
}

// ---- Internal helpers ----

/// Dispatch a scroll action (scroll_up/scroll_down) to a widget and bubble up the parent chain.
/// Stops when a widget handles the action (has it in key_bindings).
fn dispatch_scroll_action(target: WidgetId, action: &str, ctx: &AppContext) {
    use crate::event::dispatch::collect_parent_chain;
    let chain = collect_parent_chain(target, ctx);
    for &id in &chain {
        if let Some(widget) = ctx.arena.get(id) {
            // Check if this widget has a scroll action binding
            if widget.key_bindings().iter().any(|kb| kb.action == action) {
                widget.on_action(action, ctx);
                return;
            }
        }
    }
}

/// DFS traversal of widget subtree (pre-order).
fn collect_subtree_dfs(root: WidgetId, ctx: &AppContext) -> Vec<WidgetId> {
    let mut result = Vec::new();
    let mut stack = vec![root];
    while let Some(id) = stack.pop() {
        result.push(id);
        if let Some(children) = ctx.children.get(id) {
            for &child in children.iter().rev() {
                stack.push(child);
            }
        }
    }
    result
}

/// Walk the active screen subtree in DFS order and call each widget's render().
fn render_widget_tree(
    screen_id: WidgetId,
    ctx: &AppContext,
    bridge: &TaffyBridge,
    frame: &mut Frame,
) {
    use crate::css::types::{BorderStyle as TcssBorder, TcssColor};

    let buf_area = frame.area();
    let dfs_ids = collect_subtree_dfs(screen_id, ctx);
    for id in dfs_ids {
        if let Some(raw_rect) = bridge.rect_for(id) {
            // Clamp to terminal buffer bounds — Taffy may compute rects that extend
            // beyond the screen (e.g. content overflow), which would panic in ratatui.
            let rect = raw_rect.intersection(buf_area);
            if rect.width > 0 && rect.height > 0 {
                // Overlay widgets (context menu, tooltips) skip paint_chrome
                // to avoid erasing the underlying screen content.
                let is_overlay = ctx.arena.get(id).map(|w| w.is_overlay()).unwrap_or(false);
                if is_overlay {
                    if let Some(widget) = ctx.arena.get(id) {
                        widget.render(ctx, rect, frame.buffer_mut());
                    }
                    continue;
                }

                // Paint background + borders from computed CSS, get inner content area
                let content_area = if let Some(cs) = ctx.computed_styles.get(id) {
                    let is_focused = ctx.focused_widget == Some(id);
                    let is_hovered = ctx.hovered_widget == Some(id);

                    // Check widget-driven border color override (e.g. Input invalid → red)
                    let widget_border_override =
                        ctx.arena.get(id).and_then(|w| w.border_color_override());

                    if let Some((r, g, b)) = widget_border_override {
                        // Widget override takes highest priority (invalid state)
                        let mut override_cs = cs.clone();
                        override_cs.color = TcssColor::Rgb(r, g, b);
                        paint_chrome(&override_cs, rect, frame.buffer_mut())
                    } else if is_focused && cs.border != TcssBorder::None {
                        // Focused widget WITH border — upgrade border color to accent
                        let mut focused_cs = cs.clone();
                        focused_cs.color = TcssColor::Rgb(0, 255, 163); // accent green
                                                                        // Keep tall/mcgugan borders as-is; upgrade others to heavy
                        if cs.border != TcssBorder::Tall && cs.border != TcssBorder::McguganBox {
                            focused_cs.border = TcssBorder::Heavy;
                        }
                        paint_chrome(&focused_cs, rect, frame.buffer_mut())
                    } else if is_focused && cs.border == TcssBorder::None {
                        // Focused widget WITHOUT border — subtle left-edge accent bar.
                        // Don't tint the entire foreground (jarring on large content areas like Log).
                        let content = paint_chrome(cs, rect, frame.buffer_mut());
                        if rect.height > 0 {
                            let fg = ratatui::style::Color::Rgb(0, 255, 163);
                            for y in rect.y..rect.y + rect.height {
                                if let Some(cell) = frame.buffer_mut().cell_mut((rect.x, y)) {
                                    cell.set_symbol("\u{258E}"); // ▎ left quarter block
                                    cell.set_fg(fg);
                                }
                            }
                        }
                        content
                    } else if is_hovered && cs.border != TcssBorder::None {
                        // Hovered widget — lighten the border color for subtle feedback
                        let mut hover_cs = cs.clone();
                        hover_cs.color = TcssColor::Rgb(100, 180, 255); // light blue hover tint
                        paint_chrome(&hover_cs, rect, frame.buffer_mut())
                    } else {
                        paint_chrome(cs, rect, frame.buffer_mut())
                    }
                } else {
                    rect
                };
                if let Some(widget) = ctx.arena.get(id) {
                    widget.render(ctx, content_area, frame.buffer_mut());
                }
                // Draw loading overlay if this widget is in loading state
                if ctx.loading_widgets.borrow().contains_key(id) {
                    crate::widget::loading_indicator::draw_loading_spinner_overlay(
                        rect,  // Use full rect (including borders) for overlay
                        frame.buffer_mut(),
                        ctx.spinner_tick.get(),
                        ctx.skip_animations,
                    );
                }
            }
        }
    }

    // Render toast notifications (below active_overlay so CommandPalette wins z-order)
    {
        let toasts = ctx.toast_entries.borrow();
        if !toasts.is_empty() {
            crate::widget::toast::render_toasts(&toasts, buf_area, frame.buffer_mut(), &ctx.theme);
        }
    }

    // Paint the floating overlay (context menu, etc.) last, on top of everything
    if let Some(ref overlay) = *ctx.active_overlay.borrow() {
        overlay.render(ctx, buf_area, frame.buffer_mut());
    }
}
