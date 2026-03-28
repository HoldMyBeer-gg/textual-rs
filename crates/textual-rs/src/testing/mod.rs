//! Headless testing infrastructure for automated UI tests.

pub mod assertions;
pub mod pilot;

use ratatui::backend::TestBackend;
use ratatui::Terminal;

use crate::app::App;
use crate::event::AppEvent;
use crate::widget::context::AppContext;
use crate::widget::Widget;

pub use pilot::Pilot;

/// Headless test harness. Creates an App with TestBackend for automated testing.
///
/// `TestApp` does NOT run the full async event loop. Instead, events are processed
/// synchronously via `process_event`, allowing tests to control timing precisely.
///
/// # Example
/// ```ignore
/// use textual_rs::testing::TestApp;
/// let test_app = TestApp::new(80, 24, || Box::new(MyScreen));
/// ```
pub struct TestApp {
    pub(crate) app: App,
    pub(crate) terminal: Terminal<TestBackend>,
    /// Kept alive so the channel remains open; used by tests that want to inject events.
    #[allow(dead_code)]
    pub(crate) tx: flume::Sender<AppEvent>,
    pub(crate) rx: flume::Receiver<AppEvent>,
}

impl TestApp {
    /// Create a new TestApp with the given root screen factory.
    ///
    /// Initializes the reactive runtime (safe to call multiple times), mounts the
    /// root screen, and renders the initial frame.
    pub fn new(cols: u16, rows: u16, factory: impl FnOnce() -> Box<dyn Widget> + 'static) -> Self {
        // Init reactive runtime — safe to call multiple times, subsequent calls are no-ops.
        let _ = any_spawner::Executor::init_tokio();

        let mut app = App::new_bare(factory);
        // Skip animations in tests for deterministic rendering
        app.set_skip_animations(true);

        let (tx, rx) = flume::unbounded::<AppEvent>();
        app.set_event_tx(tx.clone());

        let backend = TestBackend::new(cols, rows);
        let mut terminal = Terminal::new(backend).expect("failed to create TestBackend terminal");

        // Mount root screen and render initial frame
        app.mount_root_screen();
        app.render_to_terminal(&mut terminal)
            .expect("failed initial render");

        TestApp {
            app,
            terminal,
            tx,
            rx,
        }
    }

    /// Create a TestApp WITH built-in CSS (for tests that need proper widget layout).
    pub fn new_styled(
        cols: u16,
        rows: u16,
        css: &str,
        factory: impl FnOnce() -> Box<dyn Widget> + 'static,
    ) -> Self {
        let _ = any_spawner::Executor::init_tokio();
        let mut app = App::new(factory).with_css(css);
        app.set_skip_animations(true);
        let (tx, rx) = flume::unbounded::<AppEvent>();
        app.set_event_tx(tx.clone());
        let backend = TestBackend::new(cols, rows);
        let mut terminal = Terminal::new(backend).expect("failed to create TestBackend terminal");
        app.mount_root_screen();
        app.render_to_terminal(&mut terminal)
            .expect("failed initial render");
        TestApp {
            app,
            terminal,
            tx,
            rx,
        }
    }

    /// Get a Pilot for sending simulated input events.
    pub fn pilot(&mut self) -> Pilot<'_> {
        Pilot::new(self)
    }

    /// Access the AppContext for state assertions (focus, widget ids, etc.).
    pub fn ctx(&self) -> &AppContext {
        self.app.ctx()
    }

    /// Access the rendered buffer for row-level assertions.
    pub fn buffer(&self) -> &ratatui::buffer::Buffer {
        self.terminal.backend().buffer()
    }

    /// Access the TestBackend directly (implements Display for insta snapshots).
    pub fn backend(&self) -> &TestBackend {
        self.terminal.backend()
    }

    /// Inject a key event without draining the message queue.
    ///
    /// Used in tests that need to inspect the raw message queue immediately after
    /// key dispatch (e.g., to verify a specific message was posted before bubbling).
    pub fn inject_key_event(&mut self, key: crossterm::event::KeyEvent) {
        self.app.handle_key_event(key);
    }

    /// Drain the message queue explicitly (e.g., after inject_key_event).
    pub fn drain_messages(&self) {
        self.app.drain_message_queue();
    }

    /// Process a single event synchronously and re-render.
    ///
    /// Used by Pilot to inject input. Can also be called directly for precise control.
    pub fn process_event(&mut self, event: AppEvent) {
        match &event {
            AppEvent::Key(k) if k.kind == crossterm::event::KeyEventKind::Press => {
                let k = *k;
                self.app.handle_key_event(k);
            }
            AppEvent::Mouse(m) => {
                let m = *m;
                self.app.handle_mouse_event(m);
            }
            AppEvent::RenderRequest | AppEvent::Resize(_, _) => {
                // Handled by re-render below
            }
            _ => {}
        }
        self.app.drain_message_queue();
        self.app.process_deferred_screens();
        self.app
            .render_to_terminal(&mut self.terminal)
            .expect("failed to render in process_event");
    }
}
