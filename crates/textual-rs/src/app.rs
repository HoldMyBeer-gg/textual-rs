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
Button { border: heavy; min-width: 16; height: 3; }
Checkbox { height: 1; }
Collapsible { min-height: 1; }
DataTable { border: rounded; min-height: 5; }
Footer { height: 1; }
Header { height: 1; }
Horizontal { layout-direction: horizontal; }
Input { border: rounded; height: 3; }
Label { min-height: 1; }
ListView { min-height: 3; flex-grow: 1; }
Log { min-height: 3; flex-grow: 1; }
Markdown { min-height: 3; }
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
use crate::widget::tree::{advance_focus, advance_focus_backward, clear_dirty_subtree, pop_screen, push_screen};
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
        let (worker_tx, worker_rx) = flume::unbounded::<(WidgetId, Box<dyn std::any::Any + Send>)>();
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

        // Main event loop — select! between app events and worker results
        loop {
            tokio::select! {
                event = rx.recv_async() => {
                    match event {
                        // Ignore non-press key events (release, repeat on some platforms)
                        Ok(AppEvent::Key(k)) if k.kind != KeyEventKind::Press => {}

                        Ok(AppEvent::Key(k)) => {
                            // 0. Ctrl+P: open command palette
                            if k.code == KeyCode::Char('p')
                                && k.modifiers.contains(KeyModifiers::CONTROL)
                            {
                                let commands = self.command_registry.discover_all(&self.ctx);
                                let palette = crate::command::CommandPalette::new(commands);
                                self.ctx.push_screen_deferred(Box::new(palette));
                                self.process_deferred_screens();
                                // Auto-focus the palette widget on the new overlay screen
                                advance_focus(&mut self.ctx);
                                self.full_render_pass(&mut terminal)?;
                                continue;
                            }

                            // 1. Check global quit bindings first
                            if k.code == KeyCode::Char('q')
                                || (k.code == KeyCode::Char('c')
                                    && k.modifiers.contains(KeyModifiers::CONTROL))
                            {
                                break;
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

                            // 3. If not handled by binding, dispatch as key event to focused widget, then bubble
                            if !handled {
                                if let Some(focused_id) = self.ctx.focused_widget {
                                    dispatch_message(focused_id, &k, &self.ctx);
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
                                                    .and_then(|hm| {
                                                        // Use first cell of focused widget as anchor
                                                        None::<(u16, u16)> // fallback below
                                                    })
                                                    .unwrap_or((0, 0));
                                                let overlay = crate::widget::context_menu::ContextMenuOverlay::new(
                                                    items, Some(focused_id), ax, ay,
                                                );
                                                self.ctx.push_screen_deferred(Box::new(overlay));
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
                            self.process_deferred_screens();
                            self.full_render_pass(&mut terminal)?;
                        }

                        Ok(AppEvent::Mouse(m)) => {
                            use crossterm::event::{MouseEventKind, MouseButton};
                            match m.kind {
                                MouseEventKind::Down(MouseButton::Right) => {
                                    // Right-click: spawn context menu if widget provides items
                                    if let Some(ref hit_map) = self.hit_map {
                                        if let Some(target_id) = hit_map.hit_test(m.column, m.row) {
                                            if let Some(widget) = self.ctx.arena.get(target_id) {
                                                let items = widget.context_menu_items();
                                                if !items.is_empty() {
                                                    let overlay = crate::widget::context_menu::ContextMenuOverlay::new(
                                                        items,
                                                        Some(target_id),
                                                        m.column,
                                                        m.row,
                                                    );
                                                    self.ctx.push_screen_deferred(Box::new(overlay));
                                                    self.process_deferred_screens();
                                                    self.full_render_pass(&mut terminal)?;
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
                                            self.process_deferred_screens();
                                            self.full_render_pass(&mut terminal)?;
                                        }
                                    }
                                }
                                MouseEventKind::ScrollDown
                                | MouseEventKind::ScrollUp => {
                                    let action = if m.kind == MouseEventKind::ScrollUp {
                                        "scroll_up"
                                    } else {
                                        "scroll_down"
                                    };
                                    if let Some(ref hit_map) = self.hit_map {
                                        if let Some(target_id) = hit_map.hit_test(m.column, m.row) {
                                            // Dispatch as action, bubbling up the parent chain
                                            // so scrollable containers (Log, ScrollView, ListView)
                                            // respond to mouse wheel.
                                            dispatch_scroll_action(target_id, action, &self.ctx);
                                            self.drain_message_queue();
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
                        self.process_deferred_screens();
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
        if self.ctx.screen_stack.is_empty() {
            return Ok(());
        }

        let size = terminal.size()?;

        // Process ALL screens in the stack (bottom-to-top) so overlays
        // render on top of the underlying screen content.
        for &screen_id in &self.ctx.screen_stack.clone() {
            apply_cascade_to_tree(screen_id, &self.stylesheets, &mut self.ctx);

            if self.needs_full_sync {
                self.bridge.sync_subtree(screen_id, &self.ctx);
            } else {
                self.bridge.sync_dirty_subtree(screen_id, &self.ctx);
            }

            self.bridge.compute_layout(screen_id, size.width, size.height, &self.ctx);
            clear_dirty_subtree(screen_id, &mut self.ctx);
        }
        self.needs_full_sync = false;

        // Build hit map from the TOP screen only (events go to the topmost overlay)
        let top_screen_id = *self.ctx.screen_stack.last().unwrap();
        let dfs_ids = collect_subtree_dfs(top_screen_id, &self.ctx);
        self.hit_map = Some(MouseHitMap::build(&dfs_ids, self.bridge.layout_cache()));

        // Render all screens bottom-to-top in a single frame
        let ctx_ref = &self.ctx;
        let bridge_ref = &self.bridge;
        let screen_stack = self.ctx.screen_stack.clone();
        terminal.draw(|frame| {
            for &screen_id in &screen_stack {
                render_widget_tree(screen_id, ctx_ref, bridge_ref, frame);
            }
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
        // 0. Ctrl+P: open command palette
        if k.code == KeyCode::Char('p') && k.modifiers.contains(KeyModifiers::CONTROL) {
            let commands = self.command_registry.discover_all(&self.ctx);
            let palette = crate::command::CommandPalette::new(commands);
            self.ctx.push_screen_deferred(Box::new(palette));
            self.process_deferred_screens();
            // Auto-focus the palette on the new screen
            advance_focus(&mut self.ctx);
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

        // 2. If not handled by binding, dispatch as key event to focused widget, then bubble
        if !handled {
            if let Some(focused_id) = self.ctx.focused_widget {
                dispatch_message(focused_id, &k, &self.ctx);
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
        use crossterm::event::MouseEventKind;
        match m.kind {
            MouseEventKind::Down(_) => {
                if let Some(ref hit_map) = self.hit_map {
                    if let Some(target_id) = hit_map.hit_test(m.column, m.row) {
                        dispatch_message(target_id, &m, &self.ctx);
                    }
                }
            }
            MouseEventKind::ScrollDown | MouseEventKind::ScrollUp => {
                let action = if m.kind == MouseEventKind::ScrollUp {
                    "scroll_up"
                } else {
                    "scroll_down"
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
        // Process pops first, then pushes (pop old modal before pushing new one)
        let pops = self.ctx.pending_screen_pops.get();
        if pops > 0 {
            self.ctx.pending_screen_pops.set(0);
            for _ in 0..pops {
                pop_screen(&mut self.ctx);
            }
        }

        let pushes: Vec<Box<dyn Widget>> = self.ctx.pending_screen_pushes.borrow_mut().drain(..).collect();
        for screen in pushes {
            push_screen(screen, &mut self.ctx);
        }

        // Process pending recompositions (e.g. tab switching)
        let recompose_ids: Vec<WidgetId> = self.ctx.pending_recompose.borrow_mut().drain(..).collect();
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
fn render_widget_tree(screen_id: WidgetId, ctx: &AppContext, bridge: &TaffyBridge, frame: &mut Frame) {
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
                    let widget_border_override = ctx.arena.get(id)
                        .and_then(|w| w.border_color_override());

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
            }
        }
    }
}
