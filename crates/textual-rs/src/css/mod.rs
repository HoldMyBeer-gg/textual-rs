//! TCSS (Textual CSS) styling engine: parsing, cascade, theming, and rendering.

pub mod cascade;
pub mod parser;
pub mod property;
pub mod render_style;
pub mod selector;
pub mod theme;
pub mod types;

pub use cascade::{
    apply_cascade_to_tree, resolve_cascade, stylesheet_from_css_strings, Stylesheet,
};
pub use parser::{parse_stylesheet, Rule};
pub use property::parse_declaration_block;
pub use render_style::{paint_chrome, text_style, to_ratatui_color};
pub use selector::{selector_matches, Selector, SelectorParser, Specificity};
pub use theme::{builtin_themes, default_dark_theme, default_light_theme, theme_by_name, Theme};
pub use types::*;
