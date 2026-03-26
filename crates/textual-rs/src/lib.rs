//! textual-rs — a Rust port of the Textual Python TUI framework.
//!
//! Build beautiful terminal UIs in Rust: declare widgets, style with CSS,
//! react to events, and get a polished result on any terminal.
//!
//! # Quick start
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

pub mod animation;
pub mod app;
pub mod canvas;
pub mod command;
pub mod css;
pub mod event;
pub mod layout;
pub mod reactive;
pub mod terminal;
pub mod testing;
pub mod widget;
pub mod worker;

pub use app::App;
pub use event::AppEvent;
pub use testing::TestApp;
pub use testing::pilot::Pilot;
pub use widget::{Widget, WidgetId};
// Re-export proc macros — derive macro Widget lives in a separate namespace from the Widget trait,
// so `pub use textual_rs_macros::Widget` does not conflict with `pub use widget::Widget`.
pub use textual_rs_macros::Widget;
pub use textual_rs_macros::widget_impl;
pub use widget::label::Label;
pub use widget::button::{Button, ButtonVariant};
pub use widget::checkbox::Checkbox;
pub use widget::switch::Switch;
pub use widget::input::Input;
pub use widget::radio::{RadioButton, RadioSet};
pub use widget::text_area::TextArea;
pub use widget::select::Select;
pub use widget::layout::{Vertical, Horizontal};
pub use widget::header::Header;
pub use widget::footer::Footer;
pub use widget::placeholder::Placeholder;
pub use widget::progress_bar::ProgressBar;
pub use widget::sparkline::Sparkline;
pub use widget::list_view::ListView;
pub use widget::log::Log;
pub use widget::scroll_view::ScrollView;
pub use widget::data_table::{DataTable, ColumnDef};
pub use widget::tree_view::{Tree, TreeNode};
pub use widget::tabs::{Tabs, TabbedContent};
pub use widget::collapsible::Collapsible;
pub use widget::markdown::Markdown;
pub use worker::{WorkerResult, WorkerProgress};
pub use command::{CommandPalette, CommandRegistry};
