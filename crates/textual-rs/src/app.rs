use anyhow::Result;
use crossterm::event::{Event, EventStream, KeyCode, KeyModifiers};
use futures::StreamExt;
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Layout};
use ratatui::text::Text;
use ratatui::widgets::{Block, BorderType, Borders, Paragraph};
use ratatui::{Frame, Terminal};
use tokio::runtime::Builder;
use tokio::task;
use tokio::task::LocalSet;

use crate::event::AppEvent;
use crate::terminal::{init_panic_hook, TerminalGuard};

/// Root application entry point.
/// In Phase 1 this is a skeleton that drives the event loop.
pub struct App;

impl App {
    /// Create a new App instance.
    pub fn new() -> Self {
        App
    }

    /// Run the application. Blocks the calling thread until the user quits.
    /// Creates its own single-threaded Tokio runtime internally per D-07.
    pub fn run(&self) -> Result<()> {
        init_panic_hook();
        let rt = Builder::new_current_thread().enable_all().build()?;
        let local = LocalSet::new();
        local.block_on(&rt, self.run_async())
    }

    async fn run_async(&self) -> Result<()> {
        let _guard = TerminalGuard::new()?;
        let backend = CrosstermBackend::new(std::io::stdout());
        let mut terminal = Terminal::new(backend)?;

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

        // Initial render
        terminal.draw(|f| Self::render(f))?;

        // Main event loop: blocks on flume receive per D-10
        loop {
            match rx.recv_async().await {
                Ok(AppEvent::Key(k)) if k.code == KeyCode::Char('q') => break,
                Ok(AppEvent::Key(k))
                    if k.code == KeyCode::Char('c')
                        && k.modifiers.contains(KeyModifiers::CONTROL) =>
                {
                    break;
                }
                Ok(AppEvent::Resize(_, _)) => {
                    terminal.draw(|f| Self::render(f))?;
                }
                Ok(_) => {}
                Err(_) => break, // channel closed
            }
        }

        Ok(())
    }

    fn render(f: &mut Frame) {
        let area = f.area();

        // Center a 40x10 box using Fill constraints
        let vertical = Layout::vertical([
            Constraint::Fill(1),
            Constraint::Length(10),
            Constraint::Fill(1),
        ])
        .split(area);
        let horizontal = Layout::horizontal([
            Constraint::Fill(1),
            Constraint::Length(40),
            Constraint::Fill(1),
        ])
        .split(vertical[1]);

        let block = Block::default()
            .title("textual-rs")
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded);

        let para = Paragraph::new(Text::raw("Hello from textual-rs!"))
            .block(block)
            .centered();

        f.render_widget(para, horizontal[1]);
    }
}
