#![deny(missing_docs)]
//! textual-rs — a Rust port of the [Textual](https://textual.textualize.io) Python TUI framework.
//!
//! Build beautiful terminal UIs in Rust: declare widgets, style with CSS,
//! react to events, and get a polished result on any terminal.
//!
//! # Features
//!
//! - **22+ built-in widgets** — buttons, inputs, tables, trees, tabs, markdown, and more
//! - **CSS styling engine** — TCSS stylesheets with selectors, pseudo-classes, and specificity cascade
//! - **Theme system** — 7 built-in themes with `$variable` support and lighten/darken modifiers
//! - **Reactive state** — `Reactive<T>` signals with automatic re-rendering
//! - **Flexbox & grid layout** — powered by [Taffy](https://github.com/DioxusLabs/taffy)
//! - **Async workers** — background tasks with progress streaming
//! - **Headless testing** — `TestApp` + `Pilot` for automated UI tests
//!
//! # Quick Start
//!
//! Add to your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! textual-rs = "0.3"
//! ```
//!
//! Create a minimal application:
//!
//! ```no_run
//! use textual_rs::{App, Widget, Label, Header, Footer};
//! use textual_rs::widget::context::AppContext;
//! use ratatui::{buffer::Buffer, layout::Rect};
//!
//! struct MyScreen;
//! impl Widget for MyScreen {
//!     fn widget_type_name(&self) -> &'static str { "MyScreen" }
//!     fn compose(&self) -> Vec<Box<dyn Widget>> {
//!         vec![
//!             Box::new(Header::new("My App")),
//!             Box::new(Label::new("Hello, world!")),
//!             Box::new(Footer),
//!         ]
//!     }
//!     fn render(&self, _: &AppContext, _: Rect, _: &mut Buffer) {}
//! }
//!
//! fn main() -> anyhow::Result<()> {
//!     App::new(|| Box::new(MyScreen)).run()
//! }
//! ```
//!
//! Add CSS styling with theme variables:
//!
//! ```no_run
//! # use textual_rs::{App, Widget, Label, Header, Footer};
//! # use textual_rs::widget::context::AppContext;
//! # use ratatui::{buffer::Buffer, layout::Rect};
//! # struct MyScreen;
//! # impl Widget for MyScreen {
//! #     fn widget_type_name(&self) -> &'static str { "MyScreen" }
//! #     fn render(&self, _: &AppContext, _: Rect, _: &mut Buffer) {}
//! # }
//! let mut app = App::new(|| Box::new(MyScreen))
//!     .with_css("
//!         MyScreen { background: $background; color: $foreground; }
//!         Header { background: $panel; color: $primary; }
//!     ");
//! // app.run()?;
//! ```
//!
//! Press **F5** at runtime to cycle through built-in themes (textual-dark,
//! textual-light, tokyo-night, nord, gruvbox, dracula, catppuccin).
//! Use [`App::with_theme_cycle_key`] to rebind it.
//!
//! See the [User Guide](https://github.com/jabberwock/textual-rs/blob/master/docs/guide.md)
//! and [CSS Reference](https://github.com/jabberwock/textual-rs/blob/master/docs/css-reference.md)
//! for full documentation.

pub mod animation;
pub mod app;
pub mod canvas;
pub mod command;
pub mod css;
pub mod event;
pub mod hyperlink;
pub mod layout;
pub mod reactive;
pub mod terminal;
pub mod testing;
pub mod widget;
pub mod worker;

pub use app::App;
pub use event::AppEvent;
pub use hyperlink::{LinkedLine, LinkedSpan};
pub use testing::pilot::Pilot;
pub use testing::TestApp;
pub use widget::{Widget, WidgetId};
// Re-export proc macros — derive macro Widget lives in a separate namespace from the Widget trait,
// so `pub use textual_rs_macros::Widget` does not conflict with `pub use widget::Widget`.
pub use command::{CommandPalette, CommandRegistry};
pub use terminal::{ColorDepth, RenderingQuality, TerminalCaps};
pub use textual_rs_macros::widget_impl;
pub use textual_rs_macros::Widget;
pub use widget::button::{Button, ButtonVariant};
pub use widget::checkbox::Checkbox;
pub use widget::collapsible::Collapsible;
pub use widget::data_table::{ColumnDef, DataTable};
pub use widget::footer::Footer;
pub use widget::header::Header;
pub use widget::input::Input;
pub use widget::label::Label;
pub use widget::layout::{Horizontal, Vertical};
pub use widget::list_view::ListView;
pub use widget::loading_indicator::LoadingIndicator;
pub use widget::log::Log;
pub use widget::markdown::Markdown;
pub use widget::placeholder::Placeholder;
pub use widget::progress_bar::ProgressBar;
pub use widget::radio::{RadioButton, RadioSet};
pub use widget::rich_log::RichLog;
pub use widget::screen::ModalScreen;
pub use widget::scroll_region::{ScrollContent, ScrollRegion};
pub use widget::scroll_view::ScrollView;
pub use widget::select::Select;
pub use widget::sparkline::Sparkline;
pub use widget::switch::Switch;
pub use widget::tabs::{TabbedContent, Tabs};
pub use widget::text_area::TextArea;
pub use widget::tree_view::{Tree, TreeNode};
pub use worker::{WorkerProgress, WorkerResult};
