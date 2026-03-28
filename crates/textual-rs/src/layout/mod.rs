//! Layout engine: Taffy-backed flexbox/grid bridge, hit testing, and style mapping.

pub mod bridge;
pub mod hit_map;
pub mod style_map;

pub use bridge::TaffyBridge;
pub use hit_map::MouseHitMap;
pub use style_map::taffy_style_from_computed;

#[cfg(test)]
mod tests;
