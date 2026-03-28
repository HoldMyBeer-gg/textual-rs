//! Maps ComputedStyle values to Taffy style objects for layout computation.

use crate::css::types::*;
use taffy::prelude::*;

/// Convert a `ComputedStyle` (TCSS) to a `taffy::Style`.
///
/// Dock layouts are handled via `Position::Absolute` with inset pinning (per RESEARCH D-06).
pub fn taffy_style_from_computed(s: &ComputedStyle) -> taffy::Style {
    // Handle dock layouts via Position::Absolute
    if let Some(ref dock) = s.dock {
        return dock_style(dock, s);
    }

    // If width or height uses fr units, promote to flex_grow so the widget
    // expands in its parent's flex container instead of collapsing to zero.
    let mut flex_grow = s.flex_grow;
    if let TcssDimension::Fraction(n) = s.width {
        if flex_grow == 0.0 {
            flex_grow = n;
        }
    }
    if let TcssDimension::Fraction(n) = s.height {
        if flex_grow == 0.0 {
            flex_grow = n;
        }
    }

    taffy::Style {
        display: match s.display {
            TcssDisplay::Flex => Display::Flex,
            TcssDisplay::Grid => Display::Grid,
            TcssDisplay::Block => Display::Block,
            TcssDisplay::None => Display::None,
        },
        flex_direction: match s.layout_direction {
            LayoutDirection::Vertical => FlexDirection::Column,
            LayoutDirection::Horizontal => FlexDirection::Row,
        },
        flex_grow,
        size: taffy::geometry::Size {
            width: tcss_dim_to_dimension(s.width),
            height: tcss_dim_to_dimension(s.height),
        },
        min_size: taffy::geometry::Size {
            width: tcss_dim_to_dimension(s.min_width),
            height: tcss_dim_to_dimension(s.min_height),
        },
        max_size: taffy::geometry::Size {
            width: tcss_dim_to_dimension(s.max_width),
            height: tcss_dim_to_dimension(s.max_height),
        },
        padding: tcss_sides_to_rect_lp(s.padding),
        margin: tcss_sides_to_rect_lpa(s.margin),
        border: tcss_border_to_rect(s.border),
        grid_template_columns: s
            .grid_columns
            .as_ref()
            .map(|cols| {
                cols.iter()
                    .map(|d| GridTemplateComponent::Single(tcss_dim_to_track(d)))
                    .collect()
            })
            .unwrap_or_default(),
        grid_template_rows: s
            .grid_rows
            .as_ref()
            .map(|rows| {
                rows.iter()
                    .map(|d| GridTemplateComponent::Single(tcss_dim_to_track(d)))
                    .collect()
            })
            .unwrap_or_default(),
        ..Default::default()
    }
}

/// Convert `TcssDimension` to Taffy `Dimension`.
/// Note: `Fraction` maps to `Auto` here (fr is handled in grid tracks via `tcss_dim_to_track`).
/// Note: Percent is stored as 0..100 in TCSS; Taffy uses 0.0..1.0.
fn tcss_dim_to_dimension(d: TcssDimension) -> Dimension {
    match d {
        TcssDimension::Auto => Dimension::auto(),
        TcssDimension::Length(n) => Dimension::length(n),
        TcssDimension::Percent(p) => Dimension::percent(p / 100.0),
        TcssDimension::Fraction(_) => Dimension::auto(), // fr is grid-only, handled in tracks
    }
}

/// Convert `TcssDimension` to a grid `TrackSizingFunction`.
/// - `Fraction(n)` → `fr(n)` (fractional unit)
/// - `Length(n)` → `minmax(length(n), length(n))`
/// - `Percent(p)` → `minmax(percent(p/100), percent(p/100))`
/// - `Auto` → `auto()`
fn tcss_dim_to_track(d: &TcssDimension) -> TrackSizingFunction {
    match *d {
        TcssDimension::Fraction(n) => TrackSizingFunction::from_fr(n),
        TcssDimension::Length(n) => minmax(
            MinTrackSizingFunction::length(n),
            MaxTrackSizingFunction::length(n),
        ),
        TcssDimension::Percent(p) => {
            let frac = p / 100.0;
            minmax(
                MinTrackSizingFunction::percent(frac),
                MaxTrackSizingFunction::percent(frac),
            )
        }
        TcssDimension::Auto => TrackSizingFunction::AUTO,
    }
}

/// Convert `Sides<f32>` to `taffy::geometry::Rect<LengthPercentage>` (for padding/border).
fn tcss_sides_to_rect_lp(sides: Sides<f32>) -> taffy::geometry::Rect<LengthPercentage> {
    taffy::geometry::Rect {
        left: LengthPercentage::length(sides.left),
        right: LengthPercentage::length(sides.right),
        top: LengthPercentage::length(sides.top),
        bottom: LengthPercentage::length(sides.bottom),
    }
}

/// Convert `Sides<f32>` to `taffy::geometry::Rect<LengthPercentageAuto>` (for margin).
fn tcss_sides_to_rect_lpa(sides: Sides<f32>) -> taffy::geometry::Rect<LengthPercentageAuto> {
    taffy::geometry::Rect {
        left: LengthPercentageAuto::length(sides.left),
        right: LengthPercentageAuto::length(sides.right),
        top: LengthPercentageAuto::length(sides.top),
        bottom: LengthPercentageAuto::length(sides.bottom),
    }
}

/// Convert `BorderStyle` to `taffy::geometry::Rect<LengthPercentage>`.
/// Non-None borders add 1 cell per side.
fn tcss_border_to_rect(border: BorderStyle) -> taffy::geometry::Rect<LengthPercentage> {
    let val = match border {
        BorderStyle::None => 0.0,
        _ => 1.0,
    };
    taffy::geometry::Rect {
        left: LengthPercentage::length(val),
        right: LengthPercentage::length(val),
        top: LengthPercentage::length(val),
        bottom: LengthPercentage::length(val),
    }
}

/// Build a Taffy style for a docked widget using absolute positioning.
///
/// Dock edge determines which insets are pinned to 0 and which are auto.
/// The widget's `height`/`width` from `ComputedStyle` fills the appropriate axis.
fn dock_style(edge: &DockEdge, s: &ComputedStyle) -> taffy::Style {
    let zero = LengthPercentageAuto::length(0.0);
    let auto_lpa = LengthPercentageAuto::auto();

    let inset = match edge {
        DockEdge::Top => taffy::geometry::Rect {
            top: zero,
            left: zero,
            right: zero,
            bottom: auto_lpa,
        },
        DockEdge::Bottom => taffy::geometry::Rect {
            bottom: zero,
            left: zero,
            right: zero,
            top: auto_lpa,
        },
        DockEdge::Left => taffy::geometry::Rect {
            top: zero,
            bottom: zero,
            left: zero,
            right: auto_lpa,
        },
        DockEdge::Right => taffy::geometry::Rect {
            top: zero,
            bottom: zero,
            right: zero,
            left: auto_lpa,
        },
    };

    // Size: use the widget's width/height from ComputedStyle
    let size = taffy::geometry::Size {
        width: tcss_dim_to_dimension(s.width),
        height: tcss_dim_to_dimension(s.height),
    };

    taffy::Style {
        position: Position::Absolute,
        inset,
        display: Display::Flex,
        size,
        ..Default::default()
    }
}
