//! Command palette and registry for discoverable application actions.

pub mod palette;
pub mod registry;

pub use palette::CommandPalette;
pub use registry::{Command, CommandRegistry};
