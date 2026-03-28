//! Core CSS value types used throughout the TCSS styling engine.

use std::collections::HashSet;

/// Controls how a widget participates in layout.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TcssDisplay {
    /// Flexbox layout (the default).
    Flex,
    /// CSS grid layout.
    Grid,
    /// Block layout (stacked vertically).
    Block,
    /// Widget is not rendered and takes no space.
    None,
}

/// A CSS dimension value for sizing properties.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TcssDimension {
    /// Size is determined by the layout algorithm.
    Auto,
    /// Fixed cell-count size.
    Length(f32),
    /// Size as a percentage of the parent container.
    Percent(f32),
    /// Fractional unit for proportional flex sizing.
    Fraction(f32),
}

/// Layout flow direction for flex containers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayoutDirection {
    /// Children are stacked top-to-bottom.
    Vertical,
    /// Children are arranged left-to-right.
    Horizontal,
}

/// Border rendering style for widgets.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BorderStyle {
    /// No border is drawn.
    None,
    /// Standard single-line box-drawing border.
    Solid,
    /// Rounded corners using arc box-drawing characters.
    Rounded,
    /// Heavy double-width box-drawing border.
    Heavy,
    /// Double-line box-drawing border.
    Double,
    /// ASCII-art border using `+`, `-`, and `|` characters.
    Ascii,
    /// Half-block border (▀▄▐▌) — thin frames using half-block characters.
    Tall,
    /// McGugan Box — 1/8-cell-thick borders with independent inside/outside colors.
    /// Uses one-eighth block characters (▁▔▎) for the thinnest possible border lines.
    /// The signature Textual rendering technique.
    McguganBox,
}

/// A color value in the TCSS engine.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TcssColor {
    /// Use the terminal's default color (transparent).
    Reset,
    /// An opaque RGB color.
    Rgb(u8, u8, u8),
    /// An RGBA color with an alpha channel (0–255).
    Rgba(u8, u8, u8, u8),
    /// A named color string (e.g., `"red"`).
    Named(&'static str),
}

/// A CSS pseudo-class state flag for a widget.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PseudoClass {
    /// The widget currently holds keyboard focus.
    Focus,
    /// The mouse cursor is positioned over the widget.
    Hover,
    /// The widget is disabled and not interactable.
    Disabled,
}

/// A set of active pseudo-classes for a widget.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PseudoClassSet(pub HashSet<PseudoClass>);

impl PseudoClassSet {
    /// Add a pseudo-class to the set.
    pub fn insert(&mut self, cls: PseudoClass) {
        self.0.insert(cls);
    }

    /// Remove a pseudo-class from the set.
    pub fn remove(&mut self, cls: &PseudoClass) {
        self.0.remove(cls);
    }

    /// Returns true if the given pseudo-class is active.
    pub fn contains(&self, cls: &PseudoClass) -> bool {
        self.0.contains(cls)
    }
}

/// Horizontal text alignment within a widget.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextAlign {
    /// Align text to the left edge.
    Left,
    /// Center text horizontally.
    Center,
    /// Align text to the right edge.
    Right,
}

/// Controls how content overflowing a widget's bounds is handled.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Overflow {
    /// Overflowing content is visible outside the widget bounds.
    Visible,
    /// Overflowing content is clipped.
    Hidden,
    /// A scrollbar is always shown.
    Scroll,
    /// A scrollbar appears only when content overflows.
    Auto,
}

/// Controls whether a widget is rendered.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Visibility {
    /// The widget is rendered normally.
    Visible,
    /// The widget is hidden but still occupies layout space.
    Hidden,
}

/// Inset amounts for the four sides of a widget (padding or margin).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Sides<T> {
    /// Top inset value.
    pub top: T,
    /// Right inset value.
    pub right: T,
    /// Bottom inset value.
    pub bottom: T,
    /// Left inset value.
    pub left: T,
}

impl<T: Default> Default for Sides<T> {
    fn default() -> Self {
        Sides {
            top: T::default(),
            right: T::default(),
            bottom: T::default(),
            left: T::default(),
        }
    }
}

/// An edge of the screen where a docked widget is anchored.
#[derive(Debug, Clone, PartialEq)]
pub enum DockEdge {
    /// Widget is docked to the top of its container.
    Top,
    /// Widget is docked to the bottom of its container.
    Bottom,
    /// Widget is docked to the left of its container.
    Left,
    /// Widget is docked to the right of its container.
    Right,
}

/// The resolved set of CSS properties for a widget after cascade application.
#[derive(Debug, Clone, PartialEq)]
pub struct ComputedStyle {
    /// Layout mode for this widget's children.
    pub display: TcssDisplay,
    /// Flow direction for flex layout children.
    pub layout_direction: LayoutDirection,
    /// Explicit width constraint.
    pub width: TcssDimension,
    /// Explicit height constraint.
    pub height: TcssDimension,
    /// Minimum width constraint.
    pub min_width: TcssDimension,
    /// Minimum height constraint.
    pub min_height: TcssDimension,
    /// Maximum width constraint.
    pub max_width: TcssDimension,
    /// Maximum height constraint.
    pub max_height: TcssDimension,
    /// Inner spacing between border and content.
    pub padding: Sides<f32>,
    /// Outer spacing between this widget and siblings.
    pub margin: Sides<f32>,
    /// Border drawing style.
    pub border: BorderStyle,
    /// Optional title text shown in the border.
    pub border_title: Option<String>,
    /// Foreground text color.
    pub color: TcssColor,
    /// Background fill color.
    pub background: TcssColor,
    /// Horizontal text alignment.
    pub text_align: TextAlign,
    /// Content overflow behavior.
    pub overflow: Overflow,
    /// Whether to reserve space for a scrollbar even when not scrolling.
    pub scrollbar_gutter: bool,
    /// Whether the widget is rendered or hidden.
    pub visibility: Visibility,
    /// Transparency multiplier (0.0 = fully transparent, 1.0 = opaque).
    pub opacity: f32,
    /// Edge this widget is docked to, if any.
    pub dock: Option<DockEdge>,
    /// Flex grow factor for proportional size allocation.
    pub flex_grow: f32,
    /// Grid column track definitions.
    pub grid_columns: Option<Vec<TcssDimension>>,
    /// Grid row track definitions.
    pub grid_rows: Option<Vec<TcssDimension>>,
    /// Hatch pattern background fill.
    pub hatch: Option<HatchStyle>,
    /// Keyline color for grid separators.
    pub keyline: Option<TcssColor>,
}

impl Default for ComputedStyle {
    fn default() -> Self {
        ComputedStyle {
            display: TcssDisplay::Flex,
            layout_direction: LayoutDirection::Vertical,
            width: TcssDimension::Auto,
            height: TcssDimension::Auto,
            min_width: TcssDimension::Auto,
            min_height: TcssDimension::Auto,
            max_width: TcssDimension::Auto,
            max_height: TcssDimension::Auto,
            padding: Sides::default(),
            margin: Sides::default(),
            border: BorderStyle::None,
            border_title: None,
            color: TcssColor::Reset,
            background: TcssColor::Reset,
            text_align: TextAlign::Left,
            overflow: Overflow::Visible,
            scrollbar_gutter: false,
            visibility: Visibility::Visible,
            opacity: 1.0,
            dock: None,
            flex_grow: 0.0,
            grid_columns: None,
            grid_rows: None,
            hatch: None,
            keyline: None,
        }
    }
}

/// Hatch pattern style for background fills using Unicode characters.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HatchStyle {
    /// Cross-hatch pattern (using braille dots)
    Cross,
    /// Horizontal lines
    Horizontal,
    /// Vertical lines
    Vertical,
    /// Diagonal lines going left (top-right to bottom-left)
    Left,
    /// Diagonal lines going right (top-left to bottom-right)
    Right,
}

/// A parsed CSS property value before or after theme variable resolution.
#[derive(Debug, Clone, PartialEq)]
pub enum TcssValue {
    /// A sizing dimension (length, percent, fraction, or auto).
    Dimension(TcssDimension),
    /// A resolved RGB/RGBA color.
    Color(TcssColor),
    /// A border style without an explicit color.
    Border(BorderStyle),
    /// Border style + color shorthand (e.g. "border: solid #4a4a5a")
    BorderWithColor(BorderStyle, TcssColor),
    /// A display mode value.
    Display(TcssDisplay),
    /// A text alignment value.
    TextAlign(TextAlign),
    /// An overflow behavior value.
    Overflow(Overflow),
    /// A visibility value.
    Visibility(Visibility),
    /// A bare floating-point number (opacity, flex-grow, padding cell count).
    Float(f32),
    /// A quoted string value.
    String(String),
    /// A boolean flag value.
    Bool(bool),
    /// A dock-edge placement value.
    DockEdge(DockEdge),
    /// A layout direction value.
    LayoutDirection(LayoutDirection),
    /// Shorthand with all 4 sides (padding/margin with 2+ values)
    Sides(Sides<f32>),
    /// List of dimensions (grid-template-columns/rows)
    Dimensions(Vec<TcssDimension>),
    /// Border style + unresolved theme variable (e.g. "border: tall $primary").
    /// Resolved to BorderWithColor during cascade via Theme::resolve().
    BorderWithVariable(BorderStyle, String),
    /// Unresolved theme variable reference (e.g., "primary", "accent-darken-1").
    /// Stored during parsing, resolved to Color during cascade via Theme::resolve().
    Variable(String),
    /// Hatch pattern fill (e.g., "hatch: cross")
    Hatch(HatchStyle),
    /// Keyline separator color between grid children (e.g., "keyline: $primary")
    Keyline(TcssColor),
    /// Keyline with unresolved theme variable
    KeylineVariable(String),
}

/// A single parsed CSS property-value pair.
#[derive(Debug, Clone, PartialEq)]
pub struct Declaration {
    /// The CSS property name (e.g., `"color"`, `"width"`).
    pub property: String,
    /// The parsed value for this property.
    pub value: TcssValue,
}

impl ComputedStyle {
    /// Apply a list of CSS declarations to this style, overwriting any previously set properties.
    pub fn apply_declarations(&mut self, decls: &[Declaration]) {
        for decl in decls {
            match decl.property.as_str() {
                "color" => {
                    if let TcssValue::Color(c) = decl.value {
                        self.color = c;
                    }
                }
                "background" => {
                    if let TcssValue::Color(c) = decl.value {
                        self.background = c;
                    }
                }
                "border" => match &decl.value {
                    TcssValue::Border(b) => self.border = *b,
                    TcssValue::BorderWithColor(b, c) => {
                        self.border = *b;
                        self.color = *c;
                    }
                    _ => {}
                },
                "border-title" => {
                    if let TcssValue::String(ref s) = decl.value {
                        self.border_title = Some(s.clone());
                    }
                }
                "padding" => match &decl.value {
                    TcssValue::Float(v) => {
                        self.padding = Sides {
                            top: *v,
                            right: *v,
                            bottom: *v,
                            left: *v,
                        };
                    }
                    TcssValue::Sides(s) => {
                        self.padding = *s;
                    }
                    _ => {}
                },
                "margin" => match &decl.value {
                    TcssValue::Float(v) => {
                        self.margin = Sides {
                            top: *v,
                            right: *v,
                            bottom: *v,
                            left: *v,
                        };
                    }
                    TcssValue::Sides(s) => {
                        self.margin = *s;
                    }
                    _ => {}
                },
                "width" => {
                    if let TcssValue::Dimension(d) = decl.value {
                        self.width = d;
                    }
                }
                "height" => {
                    if let TcssValue::Dimension(d) = decl.value {
                        self.height = d;
                    }
                }
                "min-width" => {
                    if let TcssValue::Dimension(d) = decl.value {
                        self.min_width = d;
                    }
                }
                "min-height" => {
                    if let TcssValue::Dimension(d) = decl.value {
                        self.min_height = d;
                    }
                }
                "max-width" => {
                    if let TcssValue::Dimension(d) = decl.value {
                        self.max_width = d;
                    }
                }
                "max-height" => {
                    if let TcssValue::Dimension(d) = decl.value {
                        self.max_height = d;
                    }
                }
                "display" => {
                    if let TcssValue::Display(d) = decl.value {
                        self.display = d;
                    }
                }
                "visibility" => {
                    if let TcssValue::Visibility(v) = decl.value {
                        self.visibility = v;
                    }
                }
                "opacity" => {
                    if let TcssValue::Float(v) = decl.value {
                        self.opacity = v;
                    }
                }
                "text-align" => {
                    if let TcssValue::TextAlign(a) = decl.value {
                        self.text_align = a;
                    }
                }
                "overflow" => {
                    if let TcssValue::Overflow(o) = decl.value {
                        self.overflow = o;
                    }
                }
                "scrollbar-gutter" => {
                    if let TcssValue::Bool(b) = decl.value {
                        self.scrollbar_gutter = b;
                    }
                }
                "dock" => {
                    if let TcssValue::DockEdge(ref d) = decl.value {
                        self.dock = Some(d.clone());
                    }
                }
                "flex-grow" => {
                    if let TcssValue::Float(v) = decl.value {
                        self.flex_grow = v;
                    }
                }
                "grid-template-columns" => {
                    if let TcssValue::Dimensions(dims) = &decl.value {
                        self.grid_columns = Some(dims.clone());
                    }
                }
                "grid-template-rows" => {
                    if let TcssValue::Dimensions(dims) = &decl.value {
                        self.grid_rows = Some(dims.clone());
                    }
                }
                "layout-direction" => {
                    if let TcssValue::LayoutDirection(d) = decl.value {
                        self.layout_direction = d;
                    }
                }
                "hatch" => {
                    if let TcssValue::Hatch(h) = decl.value {
                        self.hatch = Some(h);
                    }
                }
                "keyline" => match &decl.value {
                    TcssValue::Keyline(c) => self.keyline = Some(*c),
                    TcssValue::Color(c) => self.keyline = Some(*c),
                    _ => {}
                },
                _ => {
                    #[cfg(debug_assertions)]
                    eprintln!(
                        "[textual-rs] warning: unknown CSS property '{}' (ignored)",
                        decl.property
                    );
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn computed_style_default_values() {
        let style = ComputedStyle::default();
        assert_eq!(style.display, TcssDisplay::Flex);
        assert_eq!(style.width, TcssDimension::Auto);
        assert_eq!(style.height, TcssDimension::Auto);
        assert_eq!(style.border, BorderStyle::None);
        assert_eq!(style.color, TcssColor::Reset);
        assert_eq!(style.background, TcssColor::Reset);
        assert_eq!(style.layout_direction, LayoutDirection::Vertical);
        assert_eq!(style.opacity, 1.0);
        assert_eq!(style.flex_grow, 0.0);
        assert!(!style.scrollbar_gutter);
        assert!(style.dock.is_none());
        assert!(style.grid_columns.is_none());
        assert!(style.grid_rows.is_none());
    }

    #[test]
    fn pseudo_class_set_insert_contains_remove() {
        let mut set = PseudoClassSet::default();
        assert!(!set.contains(&PseudoClass::Focus));
        set.insert(PseudoClass::Focus);
        assert!(set.contains(&PseudoClass::Focus));
        set.insert(PseudoClass::Hover);
        assert!(set.contains(&PseudoClass::Hover));
        set.remove(&PseudoClass::Focus);
        assert!(!set.contains(&PseudoClass::Focus));
        assert!(set.contains(&PseudoClass::Hover));
        set.insert(PseudoClass::Disabled);
        assert!(set.contains(&PseudoClass::Disabled));
    }

    #[test]
    fn apply_declarations_modifies_style() {
        let mut style = ComputedStyle::default();
        let decls = vec![
            Declaration {
                property: "color".to_string(),
                value: TcssValue::Color(TcssColor::Rgb(255, 0, 0)),
            },
            Declaration {
                property: "display".to_string(),
                value: TcssValue::Display(TcssDisplay::Block),
            },
            Declaration {
                property: "opacity".to_string(),
                value: TcssValue::Float(0.5),
            },
        ];
        style.apply_declarations(&decls);
        assert_eq!(style.color, TcssColor::Rgb(255, 0, 0));
        assert_eq!(style.display, TcssDisplay::Block);
        assert_eq!(style.opacity, 0.5);
    }

    #[test]
    fn unknown_property_does_not_panic() {
        // Unknown properties should be silently ignored (with a debug warning).
        // This test verifies no panic occurs.
        let mut style = ComputedStyle::default();
        let decls = vec![
            Declaration {
                property: "nonexistent-prop".to_string(),
                value: TcssValue::Float(1.0),
            },
            Declaration {
                property: "color".to_string(),
                value: TcssValue::Color(TcssColor::Rgb(0, 255, 0)),
            },
        ];
        style.apply_declarations(&decls);
        // Known property should still be applied after the unknown one
        assert_eq!(style.color, TcssColor::Rgb(0, 255, 0));
    }
}
