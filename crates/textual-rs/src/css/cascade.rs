use crate::css::parser::{parse_stylesheet, Rule};
use crate::css::selector::{selector_matches, Specificity};
use crate::css::types::{ComputedStyle, Declaration};
use crate::widget::context::AppContext;
use crate::widget::WidgetId;

/// A parsed TCSS stylesheet with its rules.
#[derive(Debug, Clone, Default)]
pub struct Stylesheet {
    pub rules: Vec<Rule>,
}

impl Stylesheet {
    /// Parse a CSS string into a Stylesheet, returning any parse errors.
    pub fn parse(css: &str) -> (Self, Vec<String>) {
        let (rules, errors) = parse_stylesheet(css);
        (Stylesheet { rules }, errors)
    }

    /// Create an empty stylesheet.
    pub fn empty() -> Self {
        Stylesheet { rules: Vec::new() }
    }
}

/// Build a stylesheet from multiple CSS strings (e.g. default CSS from widget types).
pub fn stylesheet_from_css_strings(css_strings: &[&str]) -> (Stylesheet, Vec<String>) {
    let combined = css_strings.join("\n");
    Stylesheet::parse(&combined)
}

/// Resolve the cascade for a single widget, returning its ComputedStyle.
///
/// Stylesheets are applied in order (index 0 = lowest precedence, last = highest),
/// within each stylesheet rules are applied in specificity order (lower specificity first),
/// with source order as tiebreaker (later rule wins at equal specificity).
/// Inline styles (ctx.inline_styles) are applied last, overriding all.
pub fn resolve_cascade(
    widget_id: WidgetId,
    stylesheets: &[Stylesheet],
    ctx: &AppContext,
) -> ComputedStyle {
    // Collect matching rules with their sort keys
    // Sort key: (specificity, source_order) — lower values applied first (overridden by higher)
    let mut matched: Vec<(Specificity, usize, &Vec<Declaration>)> = Vec::new();

    for (sheet_idx, stylesheet) in stylesheets.iter().enumerate() {
        for (rule_idx, rule) in stylesheet.rules.iter().enumerate() {
            // Find the highest specificity matching selector for this rule
            let max_spec = rule
                .selectors
                .iter()
                .filter(|sel| selector_matches(sel, widget_id, ctx))
                .map(|sel| sel.specificity())
                .max();

            if let Some(spec) = max_spec {
                let source_order = sheet_idx * 100_000 + rule_idx;
                matched.push((spec, source_order, &rule.declarations));
            }
        }
    }

    // Sort ascending: lower specificity/source_order first → applied first, then overridden
    matched.sort_by(|a, b| {
        a.0.cmp(&b.0).then_with(|| a.1.cmp(&b.1))
    });

    // Apply declarations in order (later overwrites earlier)
    let mut style = ComputedStyle::default();
    for (_, _, decls) in &matched {
        style.apply_declarations(decls);
    }

    // Apply inline styles last (highest specificity)
    if let Some(inline) = ctx.inline_styles.get(widget_id) {
        style.apply_declarations(inline);
    }

    style
}

/// Walk the widget subtree rooted at `screen_id` in DFS order and compute styles for all widgets.
pub fn apply_cascade_to_tree(
    screen_id: WidgetId,
    stylesheets: &[Stylesheet],
    ctx: &mut AppContext,
) {
    // Collect all widget IDs in DFS order
    let mut stack = vec![screen_id];
    let mut order = Vec::new();

    while let Some(id) = stack.pop() {
        order.push(id);
        if let Some(children) = ctx.children.get(id) {
            // Push in reverse order so we process first child first
            for &child in children.iter().rev() {
                stack.push(child);
            }
        }
    }

    // Compute and store styles
    for id in order {
        let computed = resolve_cascade(id, stylesheets, ctx);
        ctx.computed_styles.insert(id, computed);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::css::types::{
        BorderStyle, ComputedStyle, Declaration, PseudoClass, PseudoClassSet, TcssColor,
        TcssDisplay, TcssValue,
    };
    use crate::widget::context::AppContext;
    use ratatui::{buffer::Buffer, layout::Rect};

    // Test widgets
    struct TestWidget {
        type_name: &'static str,
        classes: Vec<&'static str>,
        id: Option<&'static str>,
    }

    impl crate::widget::Widget for TestWidget {
        fn render(&self, _: &AppContext, _: Rect, _: &mut Buffer) {}
        fn widget_type_name(&self) -> &'static str { self.type_name }
        fn classes(&self) -> &[&str] { &self.classes }
        fn id(&self) -> Option<&str> { self.id }
    }

    fn btn() -> Box<dyn crate::widget::Widget> {
        Box::new(TestWidget { type_name: "Button", classes: vec![], id: None })
    }

    fn btn_with_class(cls: &'static str) -> Box<dyn crate::widget::Widget> {
        Box::new(TestWidget { type_name: "Button", classes: vec![cls], id: None })
    }

    fn btn_with_id(id: &'static str) -> Box<dyn crate::widget::Widget> {
        Box::new(TestWidget { type_name: "Button", classes: vec![], id: Some(id) })
    }

    fn setup_single_widget(w: Box<dyn crate::widget::Widget>) -> (AppContext, WidgetId) {
        let mut ctx = AppContext::new();
        let id = ctx.arena.insert(w);
        ctx.parent.insert(id, None);
        ctx.pseudo_classes.insert(id, PseudoClassSet::default());
        ctx.computed_styles.insert(id, ComputedStyle::default());
        ctx.inline_styles.insert(id, Vec::new());
        (ctx, id)
    }

    #[test]
    fn resolve_cascade_single_type_rule() {
        let (ctx, id) = setup_single_widget(btn());
        let (stylesheet, errors) = Stylesheet::parse("Button { color: #ff0000; }");
        assert!(errors.is_empty());

        let style = resolve_cascade(id, &[stylesheet], &ctx);
        assert_eq!(style.color, TcssColor::Rgb(255, 0, 0));
    }

    #[test]
    fn resolve_cascade_class_overrides_type() {
        // Button selector (type specificity) and .active selector (class specificity)
        // .active should win because it has higher specificity
        let (ctx, id) = setup_single_widget(btn_with_class("active"));
        let css = "Button { color: #ff0000; } .active { color: #0000ff; }";
        let (stylesheet, errors) = Stylesheet::parse(css);
        assert!(errors.is_empty(), "errors: {:?}", errors);

        let style = resolve_cascade(id, &[stylesheet], &ctx);
        // Class (.active) has higher specificity than type (Button), so blue should win
        assert_eq!(style.color, TcssColor::Rgb(0, 0, 255), "class should override type");
    }

    #[test]
    fn resolve_cascade_id_overrides_class() {
        let (ctx, id) = setup_single_widget(btn_with_id("main"));
        let css = ".active { color: #0000ff; } #main { color: #00ff00; }";
        // Only #main matches since this widget has no class "active"
        // But let's make a widget with both to test ID beats class
        let mut ctx2 = AppContext::new();
        let id2 = ctx2.arena.insert(Box::new(TestWidget {
            type_name: "Button",
            classes: vec!["active"],
            id: Some("main"),
        }) as Box<dyn crate::widget::Widget>);
        ctx2.parent.insert(id2, None);
        ctx2.pseudo_classes.insert(id2, PseudoClassSet::default());
        ctx2.computed_styles.insert(id2, ComputedStyle::default());
        ctx2.inline_styles.insert(id2, Vec::new());

        let (stylesheet, errors) = Stylesheet::parse(css);
        assert!(errors.is_empty());

        let style = resolve_cascade(id2, &[stylesheet], &ctx2);
        assert_eq!(style.color, TcssColor::Rgb(0, 255, 0), "ID should override class");
    }

    #[test]
    fn resolve_cascade_inline_overrides_id() {
        let mut ctx = AppContext::new();
        let id = ctx.arena.insert(Box::new(TestWidget {
            type_name: "Button",
            classes: vec!["active"],
            id: Some("main"),
        }) as Box<dyn crate::widget::Widget>);
        ctx.parent.insert(id, None);
        ctx.pseudo_classes.insert(id, PseudoClassSet::default());
        ctx.computed_styles.insert(id, ComputedStyle::default());
        // Inline style sets color to red
        ctx.inline_styles.insert(id, vec![Declaration {
            property: "color".to_string(),
            value: TcssValue::Color(TcssColor::Rgb(255, 0, 0)),
        }]);

        let css = "#main { color: #0000ff; }";
        let (stylesheet, errors) = Stylesheet::parse(css);
        assert!(errors.is_empty());

        let style = resolve_cascade(id, &[stylesheet], &ctx);
        // Inline should win over ID selector
        assert_eq!(style.color, TcssColor::Rgb(255, 0, 0), "inline should override ID");
    }

    #[test]
    fn resolve_cascade_same_specificity_later_source_wins() {
        let (ctx, id) = setup_single_widget(btn());
        // Two type selectors — same specificity; second rule (blue) should win
        let css = "Button { color: red; } Button { color: #0000ff; }";
        let (stylesheet, errors) = Stylesheet::parse(css);
        assert!(errors.is_empty(), "errors: {:?}", errors);

        let style = resolve_cascade(id, &[stylesheet], &ctx);
        assert_eq!(style.color, TcssColor::Rgb(0, 0, 255), "later source should win at equal specificity");
    }

    #[test]
    fn resolve_cascade_focus_pseudo_class_only_when_focused() {
        let mut ctx = AppContext::new();
        let id = ctx.arena.insert(btn());
        ctx.parent.insert(id, None);
        ctx.pseudo_classes.insert(id, PseudoClassSet::default()); // no focus
        ctx.computed_styles.insert(id, ComputedStyle::default());
        ctx.inline_styles.insert(id, Vec::new());

        let css = "Button { color: red; } Button:focus { color: #0000ff; }";
        let (stylesheet, errors) = Stylesheet::parse(css);
        assert!(errors.is_empty(), "errors: {:?}", errors);

        // Without focus — should be red
        let style = resolve_cascade(id, &[stylesheet.clone()], &ctx);
        assert_eq!(style.color, TcssColor::Rgb(255, 0, 0), "without focus should be red");

        // Now add focus
        let mut pcs = PseudoClassSet::default();
        pcs.insert(PseudoClass::Focus);
        ctx.pseudo_classes.insert(id, pcs);

        let style = resolve_cascade(id, &[stylesheet], &ctx);
        assert_eq!(style.color, TcssColor::Rgb(0, 0, 255), "with focus should be blue");
    }

    #[test]
    fn resolve_cascade_default_css_overridden_by_user() {
        let (ctx, id) = setup_single_widget(btn());

        // Default CSS at sheet index 0 (lowest)
        let (default_sheet, _) = Stylesheet::parse("Button { color: red; }");
        // User CSS at sheet index 1 (higher)
        let (user_sheet, _) = Stylesheet::parse("Button { color: #0000ff; }");

        let style = resolve_cascade(id, &[default_sheet, user_sheet], &ctx);
        // User stylesheet (sheet 1) should override default (sheet 0) even at equal specificity
        assert_eq!(style.color, TcssColor::Rgb(0, 0, 255), "user CSS should override default CSS");
    }

    #[test]
    fn full_roundtrip_parse_cascade_computed_style() {
        let mut ctx = AppContext::new();
        let id = ctx.arena.insert(Box::new(TestWidget {
            type_name: "Button",
            classes: vec!["active"],
            id: Some("main"),
        }) as Box<dyn crate::widget::Widget>);
        ctx.parent.insert(id, None);
        ctx.pseudo_classes.insert(id, PseudoClassSet::default());
        ctx.computed_styles.insert(id, ComputedStyle::default());
        ctx.inline_styles.insert(id, Vec::new());

        let css = r#"
            Button { color: red; display: block; }
            .active { border: rounded; }
            #main { width: 50%; }
        "#;
        let (stylesheet, errors) = Stylesheet::parse(css);
        assert!(errors.is_empty(), "parse errors: {:?}", errors);

        let style = resolve_cascade(id, &[stylesheet], &ctx);

        // Type rule applied (lowest specificity)
        assert_eq!(style.color, TcssColor::Rgb(255, 0, 0));
        assert_eq!(style.display, TcssDisplay::Block);
        // Class rule applied (middle specificity)
        assert_eq!(style.border, BorderStyle::Rounded);
        // ID rule applied (highest specificity from selector)
        assert_eq!(style.width, crate::css::types::TcssDimension::Percent(50.0));
    }

    #[test]
    fn apply_cascade_to_tree_sets_computed_styles() {
        let mut ctx = AppContext::new();
        let screen = ctx.arena.insert(Box::new(TestWidget { type_name: "Screen", classes: vec![], id: None }) as Box<dyn crate::widget::Widget>);
        let button = ctx.arena.insert(Box::new(TestWidget { type_name: "Button", classes: vec![], id: None }) as Box<dyn crate::widget::Widget>);

        ctx.parent.insert(screen, None);
        ctx.parent.insert(button, Some(screen));
        ctx.children.insert(screen, vec![button]);
        ctx.children.insert(button, vec![]);
        ctx.pseudo_classes.insert(screen, PseudoClassSet::default());
        ctx.pseudo_classes.insert(button, PseudoClassSet::default());
        ctx.computed_styles.insert(screen, ComputedStyle::default());
        ctx.computed_styles.insert(button, ComputedStyle::default());
        ctx.inline_styles.insert(screen, Vec::new());
        ctx.inline_styles.insert(button, Vec::new());

        let css = "Button { color: red; }";
        let (stylesheet, errors) = Stylesheet::parse(css);
        assert!(errors.is_empty());

        apply_cascade_to_tree(screen, &[stylesheet], &mut ctx);

        let button_style = ctx.computed_styles.get(button).unwrap();
        assert_eq!(button_style.color, TcssColor::Rgb(255, 0, 0));
    }

    #[test]
    fn stylesheet_from_css_strings_combines_multiple() {
        let css1 = "Button { color: red; }";
        let css2 = "Label { display: block; }";
        let (stylesheet, errors) = stylesheet_from_css_strings(&[css1, css2]);
        assert!(errors.is_empty(), "errors: {:?}", errors);
        assert_eq!(stylesheet.rules.len(), 2);
    }
}
