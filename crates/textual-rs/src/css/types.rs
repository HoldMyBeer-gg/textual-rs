// Placeholder for RED phase — tests reference these types but they don't compile yet
// This file will be replaced in the GREEN phase with full implementation

use std::collections::HashSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TcssDisplay {
    Flex,
    Grid,
    Block,
    None,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TcssDimension {
    Auto,
    Length(f32),
    Percent(f32),
    Fraction(f32),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayoutDirection {
    Vertical,
    Horizontal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BorderStyle {
    None,
    Solid,
    Rounded,
    Heavy,
    Double,
    Ascii,
    /// Half-block border (▀▄▐▌) — thin frames using half-block characters.
    Tall,
    /// McGugan Box — 1/8-cell-thick borders with independent inside/outside colors.
    /// Uses one-eighth block characters (▁▔▎) for the thinnest possible border lines.
    /// The signature Textual rendering technique.
    McguganBox,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TcssColor {
    Reset,
    Rgb(u8, u8, u8),
    Rgba(u8, u8, u8, u8),
    Named(&'static str),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PseudoClass {
    Focus,
    Hover,
    Disabled,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PseudoClassSet(pub HashSet<PseudoClass>);

impl PseudoClassSet {
    pub fn insert(&mut self, cls: PseudoClass) {
        self.0.insert(cls);
    }

    pub fn remove(&mut self, cls: &PseudoClass) {
        self.0.remove(cls);
    }

    pub fn contains(&self, cls: &PseudoClass) -> bool {
        self.0.contains(cls)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextAlign {
    Left,
    Center,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Overflow {
    Visible,
    Hidden,
    Scroll,
    Auto,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Visibility {
    Visible,
    Hidden,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Sides<T> {
    pub top: T,
    pub right: T,
    pub bottom: T,
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

#[derive(Debug, Clone, PartialEq)]
pub enum DockEdge {
    Top,
    Bottom,
    Left,
    Right,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ComputedStyle {
    pub display: TcssDisplay,
    pub layout_direction: LayoutDirection,
    pub width: TcssDimension,
    pub height: TcssDimension,
    pub min_width: TcssDimension,
    pub min_height: TcssDimension,
    pub max_width: TcssDimension,
    pub max_height: TcssDimension,
    pub padding: Sides<f32>,
    pub margin: Sides<f32>,
    pub border: BorderStyle,
    pub border_title: Option<String>,
    pub color: TcssColor,
    pub background: TcssColor,
    pub text_align: TextAlign,
    pub overflow: Overflow,
    pub scrollbar_gutter: bool,
    pub visibility: Visibility,
    pub opacity: f32,
    pub dock: Option<DockEdge>,
    pub flex_grow: f32,
    pub grid_columns: Option<Vec<TcssDimension>>,
    pub grid_rows: Option<Vec<TcssDimension>>,
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
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TcssValue {
    Dimension(TcssDimension),
    Color(TcssColor),
    Border(BorderStyle),
    /// Border style + color shorthand (e.g. "border: solid #4a4a5a")
    BorderWithColor(BorderStyle, TcssColor),
    Display(TcssDisplay),
    TextAlign(TextAlign),
    Overflow(Overflow),
    Visibility(Visibility),
    Float(f32),
    String(String),
    Bool(bool),
    DockEdge(DockEdge),
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
}

#[derive(Debug, Clone, PartialEq)]
pub struct Declaration {
    pub property: String,
    pub value: TcssValue,
}

impl ComputedStyle {
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
                "border" => {
                    match &decl.value {
                        TcssValue::Border(b) => self.border = *b,
                        TcssValue::BorderWithColor(b, c) => {
                            self.border = *b;
                            self.color = c.clone();
                        }
                        _ => {}
                    }
                }
                "border-title" => {
                    if let TcssValue::String(ref s) = decl.value {
                        self.border_title = Some(s.clone());
                    }
                }
                "padding" => {
                    if let TcssValue::Float(v) = decl.value {
                        self.padding = Sides { top: v, right: v, bottom: v, left: v };
                    }
                }
                "margin" => {
                    if let TcssValue::Float(v) = decl.value {
                        self.margin = Sides { top: v, right: v, bottom: v, left: v };
                    }
                }
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
                _ => {}
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
}
