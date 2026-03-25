use std::iter;

use cssparser::{ParseError, Parser, Token};

use crate::css::types::PseudoClass;
use crate::widget::context::AppContext;
use crate::widget::WidgetId;

/// TCSS selector — a single selector or combinator expression.
#[derive(Debug, Clone, PartialEq)]
pub enum Selector {
    /// Matches widgets by type name, e.g. `Button`
    Type(String),
    /// Matches widgets with a CSS class, e.g. `.active`
    Class(String),
    /// Matches widgets with a specific ID, e.g. `#sidebar`
    Id(String),
    /// Matches any widget (`*`)
    Universal,
    /// Matches widgets with a given pseudo-class, e.g. `:focus`
    PseudoClass(PseudoClass),
    /// Matches a widget that is any descendant of the left selector, e.g. `Screen Button`
    Descendant(Box<Selector>, Box<Selector>),
    /// Matches a widget that is a direct child of the left selector, e.g. `Container > Button`
    Child(Box<Selector>, Box<Selector>),
    /// Multiple simple selectors that all apply to the same element, e.g. `Button.active:focus`
    Compound(Vec<Selector>),
}

/// CSS specificity as (id_count, class_count, type_count).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Specificity(pub u32, pub u32, pub u32);

impl Selector {
    /// Calculate specificity for this selector.
    pub fn specificity(&self) -> Specificity {
        match self {
            Selector::Id(_) => Specificity(1, 0, 0),
            Selector::Class(_) => Specificity(0, 1, 0),
            Selector::PseudoClass(_) => Specificity(0, 1, 0),
            Selector::Type(_) => Specificity(0, 0, 1),
            Selector::Universal => Specificity(0, 0, 0),
            Selector::Compound(parts) => {
                let mut total = Specificity(0, 0, 0);
                for s in parts {
                    let Specificity(a, b, c) = s.specificity();
                    total = Specificity(total.0 + a, total.1 + b, total.2 + c);
                }
                total
            }
            Selector::Descendant(left, right) => {
                let Specificity(a1, b1, c1) = left.specificity();
                let Specificity(a2, b2, c2) = right.specificity();
                Specificity(a1 + a2, b1 + b2, c1 + c2)
            }
            Selector::Child(left, right) => {
                let Specificity(a1, b1, c1) = left.specificity();
                let Specificity(a2, b2, c2) = right.specificity();
                Specificity(a1 + a2, b1 + b2, c1 + c2)
            }
        }
    }
}

/// Iterate over ancestors of a widget, from immediate parent upwards.
fn ancestors(id: WidgetId, ctx: &AppContext) -> impl Iterator<Item = WidgetId> + '_ {
    iter::successors(
        ctx.parent.get(id).and_then(|p| *p),
        move |&cur| ctx.parent.get(cur).and_then(|p| *p),
    )
}

/// Test if a selector matches a specific widget in the given context.
pub fn selector_matches(sel: &Selector, id: WidgetId, ctx: &AppContext) -> bool {
    match sel {
        Selector::Universal => true,
        Selector::Type(name) => {
            ctx.arena.get(id).map_or(false, |w| w.widget_type_name() == name.as_str())
        }
        Selector::Class(cls) => {
            ctx.arena.get(id).map_or(false, |w| w.classes().contains(&cls.as_str()))
        }
        Selector::Id(expected_id) => {
            ctx.arena.get(id).map_or(false, |w| w.id() == Some(expected_id.as_str()))
        }
        Selector::PseudoClass(pc) => {
            ctx.pseudo_classes.get(id).map_or(false, |set| set.contains(pc))
        }
        Selector::Compound(parts) => parts.iter().all(|s| selector_matches(s, id, ctx)),
        Selector::Descendant(ancestor_sel, subject_sel) => {
            if !selector_matches(subject_sel, id, ctx) {
                return false;
            }
            ancestors(id, ctx).any(|anc_id| selector_matches(ancestor_sel, anc_id, ctx))
        }
        Selector::Child(parent_sel, subject_sel) => {
            if !selector_matches(subject_sel, id, ctx) {
                return false;
            }
            // Get the direct parent
            ctx.parent
                .get(id)
                .and_then(|p| *p)
                .map_or(false, |parent_id| selector_matches(parent_sel, parent_id, ctx))
        }
    }
}

/// Error type for selector parsing.
#[derive(Debug, Clone)]
pub struct SelectorParseError(pub String);

/// Parse a pseudo-class name to PseudoClass variant.
fn parse_pseudo_class_name(name: &str) -> Option<PseudoClass> {
    match name {
        "focus" => Some(PseudoClass::Focus),
        "hover" => Some(PseudoClass::Hover),
        "disabled" => Some(PseudoClass::Disabled),
        _ => None,
    }
}

/// Parse a compound selector (sequence of simple selectors without whitespace/combinator).
fn parse_compound_selector_tokens<'i>(
    input: &mut Parser<'i, '_>,
) -> Result<Option<Selector>, ParseError<'i, SelectorParseError>> {
    let mut simples: Vec<Selector> = Vec::new();

    loop {
        let state = input.state();
        let location = input.current_source_location();
        match input.next_including_whitespace() {
            Ok(Token::Ident(name)) => {
                simples.push(Selector::Type(name.to_string()));
            }
            Ok(Token::Delim('*')) => {
                simples.push(Selector::Universal);
            }
            Ok(Token::Delim('.')) => {
                let class_name = input.expect_ident_cloned().map_err(|_| {
                    location.new_custom_error(SelectorParseError("expected class name after '.'".to_string()))
                })?;
                simples.push(Selector::Class(class_name.to_string()));
            }
            Ok(Token::IDHash(id_val)) => {
                simples.push(Selector::Id(id_val.to_string()));
            }
            Ok(Token::Colon) => {
                let pc_name = input.expect_ident_cloned().map_err(|_| {
                    location.new_custom_error(SelectorParseError("expected pseudo-class name after ':'".to_string()))
                })?;
                match parse_pseudo_class_name(pc_name.as_ref()) {
                    Some(pc) => simples.push(Selector::PseudoClass(pc)),
                    None => {
                        return Err(location.new_custom_error(SelectorParseError(
                            format!("unknown pseudo-class: {}", pc_name),
                        )));
                    }
                }
            }
            // Whitespace, comma, combinator, or EOF — stop compound
            Ok(Token::WhiteSpace(_)) | Ok(Token::Comma) | Ok(Token::Delim('>')) | Err(_) => {
                input.reset(&state);
                break;
            }
            _ => {
                input.reset(&state);
                break;
            }
        }
    }

    if simples.is_empty() {
        return Ok(None);
    }
    if simples.len() == 1 {
        Ok(Some(simples.remove(0)))
    } else {
        Ok(Some(Selector::Compound(simples)))
    }
}

/// Parse a single selector (possibly with combinators) from the input.
fn parse_single_selector<'i>(
    input: &mut Parser<'i, '_>,
) -> Result<Option<Selector>, ParseError<'i, SelectorParseError>> {
    // Skip leading whitespace
    input.skip_whitespace();

    let first = match parse_compound_selector_tokens(input)? {
        Some(s) => s,
        None => return Ok(None),
    };

    let mut current = first;

    loop {
        // Check what combinator follows
        let state = input.state();
        match input.next_including_whitespace() {
            Ok(Token::WhiteSpace(_)) => {
                // Could be descendant combinator OR followed by >
                // Check if next non-whitespace is >
                input.skip_whitespace();
                let state2 = input.state();
                match input.next_including_whitespace() {
                    Ok(Token::Delim('>')) => {
                        // Descendant + > = child combinator (same as direct >)
                        input.skip_whitespace();
                        match parse_compound_selector_tokens(input)? {
                            Some(right) => {
                                current = Selector::Child(Box::new(current), Box::new(right));
                            }
                            None => break,
                        }
                    }
                    Ok(Token::Comma) | Err(_) => {
                        input.reset(&state2);
                        break;
                    }
                    _ => {
                        // Descendant combinator — reset and parse right-hand compound
                        input.reset(&state2);
                        match parse_compound_selector_tokens(input)? {
                            Some(right) => {
                                current = Selector::Descendant(Box::new(current), Box::new(right));
                            }
                            None => break,
                        }
                    }
                }
            }
            Ok(Token::Delim('>')) => {
                // Child combinator
                input.skip_whitespace();
                match parse_compound_selector_tokens(input)? {
                    Some(right) => {
                        current = Selector::Child(Box::new(current), Box::new(right));
                    }
                    None => break,
                }
            }
            Ok(Token::Comma) | Err(_) => {
                input.reset(&state);
                break;
            }
            _ => {
                input.reset(&state);
                break;
            }
        }
    }

    Ok(Some(current))
}

/// Selector parser that operates on a cssparser input stream.
pub struct SelectorParser;

impl SelectorParser {
    /// Parse a comma-separated list of selectors.
    pub fn parse_selector_list<'i>(
        input: &mut Parser<'i, '_>,
    ) -> Result<Vec<Selector>, ParseError<'i, SelectorParseError>> {
        let mut selectors = Vec::new();

        loop {
            input.skip_whitespace();
            if input.is_exhausted() {
                break;
            }

            match parse_single_selector(input)? {
                Some(sel) => selectors.push(sel),
                None => break,
            }

            input.skip_whitespace();
            if input.is_exhausted() {
                break;
            }

            // Try to consume comma for next selector
            let state = input.state();
            match input.next() {
                Ok(Token::Comma) => {
                    // Continue to next selector
                }
                _ => {
                    input.reset(&state);
                    break;
                }
            }
        }

        Ok(selectors)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::css::types::{ComputedStyle, PseudoClassSet};
    use crate::widget::context::AppContext;
    use ratatui::{buffer::Buffer, layout::Rect};

    fn parse_selector(input: &str) -> Vec<Selector> {
        let mut parser_input = cssparser::ParserInput::new(input);
        let mut parser = cssparser::Parser::new(&mut parser_input);
        SelectorParser::parse_selector_list(&mut parser).expect("parse failed")
    }

    // Minimal widget for tests
    struct TypeWidget(&'static str);
    impl crate::widget::Widget for TypeWidget {
        fn render(&self, _: &AppContext, _: Rect, _: &mut Buffer) {}
        fn widget_type_name(&self) -> &'static str { self.0 }
    }

    fn make_single_widget_ctx(w: Box<dyn crate::widget::Widget>) -> (AppContext, WidgetId) {
        let mut ctx = AppContext::new();
        let id = ctx.arena.insert(w);
        ctx.parent.insert(id, None);
        ctx.pseudo_classes.insert(id, PseudoClassSet::default());
        ctx.computed_styles.insert(id, ComputedStyle::default());
        ctx.inline_styles.insert(id, Vec::new());
        (ctx, id)
    }

    #[test]
    fn parse_type_selector() {
        let sels = parse_selector("Button");
        assert_eq!(sels, vec![Selector::Type("Button".to_string())]);
    }

    #[test]
    fn parse_class_selector() {
        let sels = parse_selector(".highlight");
        assert_eq!(sels, vec![Selector::Class("highlight".to_string())]);
    }

    #[test]
    fn parse_id_selector() {
        let sels = parse_selector("#sidebar");
        assert_eq!(sels, vec![Selector::Id("sidebar".to_string())]);
    }

    #[test]
    fn parse_compound_selector_type_class() {
        let sels = parse_selector("Button.active");
        assert_eq!(
            sels,
            vec![Selector::Compound(vec![
                Selector::Type("Button".to_string()),
                Selector::Class("active".to_string()),
            ])]
        );
    }

    #[test]
    fn parse_descendant_selector() {
        let sels = parse_selector("Screen Button");
        assert_eq!(
            sels,
            vec![Selector::Descendant(
                Box::new(Selector::Type("Screen".to_string())),
                Box::new(Selector::Type("Button".to_string())),
            )]
        );
    }

    #[test]
    fn parse_child_selector() {
        let sels = parse_selector("Container > Button");
        assert_eq!(
            sels,
            vec![Selector::Child(
                Box::new(Selector::Type("Container".to_string())),
                Box::new(Selector::Type("Button".to_string())),
            )]
        );
    }

    #[test]
    fn parse_pseudo_class_selector() {
        let sels = parse_selector("Button:focus");
        assert_eq!(
            sels,
            vec![Selector::Compound(vec![
                Selector::Type("Button".to_string()),
                Selector::PseudoClass(PseudoClass::Focus),
            ])]
        );
    }

    #[test]
    fn parse_universal_selector() {
        let sels = parse_selector("*");
        assert_eq!(sels, vec![Selector::Universal]);
    }

    #[test]
    fn selector_matches_type_correct() {
        let (ctx, id) = make_single_widget_ctx(Box::new(TypeWidget("Button")));
        assert!(selector_matches(&Selector::Type("Button".to_string()), id, &ctx));
    }

    #[test]
    fn selector_matches_type_wrong() {
        let (ctx, id) = make_single_widget_ctx(Box::new(TypeWidget("Label")));
        assert!(!selector_matches(&Selector::Type("Button".to_string()), id, &ctx));
    }

    #[test]
    fn selector_matches_descendant() {
        let mut ctx = AppContext::new();
        let screen = ctx.arena.insert(Box::new(TypeWidget("Screen")) as Box<dyn crate::widget::Widget>);
        let button = ctx.arena.insert(Box::new(TypeWidget("Button")) as Box<dyn crate::widget::Widget>);
        ctx.parent.insert(screen, None);
        ctx.parent.insert(button, Some(screen));
        ctx.pseudo_classes.insert(screen, PseudoClassSet::default());
        ctx.pseudo_classes.insert(button, PseudoClassSet::default());
        ctx.children.insert(screen, vec![button]);

        let sel = Selector::Descendant(
            Box::new(Selector::Type("Screen".to_string())),
            Box::new(Selector::Type("Button".to_string())),
        );
        assert!(selector_matches(&sel, button, &ctx));
    }

    #[test]
    fn selector_matches_child_non_parent_returns_false() {
        // Screen -> Container -> Button; test "Screen > Button" against Button (should be false)
        let mut ctx = AppContext::new();
        let screen = ctx.arena.insert(Box::new(TypeWidget("Screen")) as Box<dyn crate::widget::Widget>);
        let container = ctx.arena.insert(Box::new(TypeWidget("Container")) as Box<dyn crate::widget::Widget>);
        let button = ctx.arena.insert(Box::new(TypeWidget("Button")) as Box<dyn crate::widget::Widget>);
        ctx.parent.insert(screen, None);
        ctx.parent.insert(container, Some(screen));
        ctx.parent.insert(button, Some(container));
        ctx.pseudo_classes.insert(screen, PseudoClassSet::default());
        ctx.pseudo_classes.insert(container, PseudoClassSet::default());
        ctx.pseudo_classes.insert(button, PseudoClassSet::default());

        let sel = Selector::Child(
            Box::new(Selector::Type("Screen".to_string())),
            Box::new(Selector::Type("Button".to_string())),
        );
        // Screen is NOT the direct parent of Button, so should be false
        assert!(!selector_matches(&sel, button, &ctx));
    }

    #[test]
    fn selector_matches_pseudo_class_focus() {
        let mut ctx = AppContext::new();
        let btn = ctx.arena.insert(Box::new(TypeWidget("Button")) as Box<dyn crate::widget::Widget>);
        ctx.parent.insert(btn, None);
        let mut pcs = PseudoClassSet::default();
        pcs.insert(PseudoClass::Focus);
        ctx.pseudo_classes.insert(btn, pcs);

        assert!(selector_matches(&Selector::PseudoClass(PseudoClass::Focus), btn, &ctx));
    }

    #[test]
    fn specificity_ordering() {
        let type_spec = Selector::Type("Button".to_string()).specificity();
        let class_spec = Selector::Class("active".to_string()).specificity();
        let id_spec = Selector::Id("main".to_string()).specificity();

        assert!(type_spec < class_spec, "type should have lower specificity than class");
        assert!(class_spec < id_spec, "class should have lower specificity than id");
        assert_eq!(type_spec, Specificity(0, 0, 1));
        assert_eq!(class_spec, Specificity(0, 1, 0));
        assert_eq!(id_spec, Specificity(1, 0, 0));
    }
}
