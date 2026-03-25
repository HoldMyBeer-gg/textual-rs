use anyhow::Result;
use crossterm::event::{Event, EventStream, KeyCode, KeyModifiers};
use futures::StreamExt;
use ratatui::backend::{Backend, CrosstermBackend, TestBackend};
use ratatui::{Frame, Terminal};
use tokio::runtime::Builder;
use tokio::task;
use tokio::task::LocalSet;

use crate::css::cascade::{apply_cascade_to_tree, Stylesheet};
use crate::event::AppEvent;
use crate::layout::bridge::TaffyBridge;
use crate::layout::hit_map::MouseHitMap;
use crate::terminal::{init_panic_hook, TerminalGuard};
use crate::widget::context::AppContext;
use crate::widget::tree::{advance_focus, advance_focus_backward, clear_dirty_subtree, push_screen};
use crate::widget::{Widget, WidgetId};

/// Root application entry point.
/// Owns AppContext, TaffyBridge, and stylesheets — the three Phase 2 subsystems.
pub struct App {
    ctx: AppContext,
    bridge: TaffyBridge,
    stylesheets: Vec<Stylesheet>,
    hit_map: Option<MouseHitMap>,
    root_screen_factory: Option<Box<dyn FnOnce() -> Box<dyn Widget>>>,
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
        App {
            ctx: AppContext::new(),
            bridge: TaffyBridge::new(),
            stylesheets: Vec::new(),
            hit_map: None,
            root_screen_factory: Some(Box::new(screen_factory)),
        }
    }

    /// Builder: parse and add a TCSS stylesheet. Parse errors are logged to stderr.
    pub fn with_css(mut self, css: &str) -> Self {
        let (stylesheet, errors) = Stylesheet::parse(css);
        for err in &errors {
            eprintln!("[textual-rs] CSS parse error: {}", err);
        }
        self.stylesheets.push(stylesheet);
        self
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
        let _guard = TerminalGuard::new()?;
        let backend = CrosstermBackend::new(std::io::stdout());
        let mut terminal = Terminal::new(backend)?;

        // Mount root screen
        if let Some(factory) = self.root_screen_factory.take() {
            let root_screen = factory();
            push_screen(root_screen, &mut self.ctx);
        }

        // Initial render
        self.full_render_pass(&mut terminal)?;

        let (tx, rx) = flume::unbounded::<AppEvent>();

        // Spawn EventStream reader task on LocalSet (does not need Send)
        task::spawn_local(async move {
            let mut stream = EventStream::new();
            while let Some(Ok(event)) = stream.next().await {
                let app_event = match event {
                    Event::Key(k) => Some(AppEvent::Key(k)),
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

        // Main event loop
        loop {
            match rx.recv_async().await {
                Ok(AppEvent::Key(k)) if k.code == KeyCode::Char('q') => break,
                Ok(AppEvent::Key(k))
                    if k.code == KeyCode::Char('c')
                        && k.modifiers.contains(KeyModifiers::CONTROL) =>
                {
                    break;
                }
                Ok(AppEvent::Key(k)) if k.code == KeyCode::Tab => {
                    if k.modifiers.contains(KeyModifiers::SHIFT) {
                        advance_focus_backward(&mut self.ctx);
                    } else {
                        advance_focus(&mut self.ctx);
                    }
                    self.full_render_pass(&mut terminal)?;
                }
                Ok(AppEvent::Resize(_, _)) => {
                    self.full_render_pass(&mut terminal)?;
                }
                Ok(_) => {}
                Err(_) => break, // channel closed
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
            None => return Ok(()), // no screen mounted yet
        };

        // a. Apply CSS cascade
        apply_cascade_to_tree(screen_id, &self.stylesheets, &mut self.ctx);

        // b. Sync layout tree
        self.bridge.sync_dirty_subtree(screen_id, &self.ctx);

        // c. Compute layout
        let size = terminal.size()?;
        self.bridge.compute_layout(screen_id, size.width, size.height);

        // d. Clear dirty flags
        clear_dirty_subtree(screen_id, &mut self.ctx);

        // e. Build hit map (DFS widget ids)
        let dfs_ids = collect_subtree_dfs(screen_id, &self.ctx);
        self.hit_map = Some(MouseHitMap::build(&dfs_ids, self.bridge.layout_cache()));

        // f. Render
        let ctx_ref = &self.ctx;
        let bridge_ref = &self.bridge;
        terminal.draw(|frame| {
            render_widget_tree(screen_id, ctx_ref, bridge_ref, frame);
        })?;

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
        self.bridge.compute_layout(screen_id, cols, rows);
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
}

// ---- Internal helpers ----

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
fn render_widget_tree(screen_id: WidgetId, ctx: &AppContext, bridge: &TaffyBridge, frame: &mut Frame) {
    let dfs_ids = collect_subtree_dfs(screen_id, ctx);
    for id in dfs_ids {
        if let Some(rect) = bridge.rect_for(id) {
            if rect.width > 0 && rect.height > 0 {
                if let Some(widget) = ctx.arena.get(id) {
                    widget.render(ctx, rect, frame.buffer_mut());
                }
            }
        }
    }
}
