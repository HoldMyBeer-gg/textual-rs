use proptest::prelude::*;
use textual_rs::css::cascade::Stylesheet;

proptest! {
    /// The CSS parser must NOT panic on any arbitrary string input.
    #[test]
    fn css_parser_never_panics(input in ".*") {
        let _ = Stylesheet::parse(&input);
    }

    /// Valid RGB color syntax must parse without errors.
    #[test]
    fn css_valid_rgb_colors_parse(
        r in 0u8..=255u8,
        g in 0u8..=255u8,
        b in 0u8..=255u8,
    ) {
        let css = format!("Widget {{ color: rgb({}, {}, {}); }}", r, g, b);
        let (_, errors) = Stylesheet::parse(&css);
        prop_assert!(
            errors.is_empty(),
            "Valid RGB color should parse without errors: rgb({},{},{}) — errors: {:?}",
            r, g, b, errors
        );
    }

    /// The CSS parser must not panic on plausible CSS-like character sequences.
    #[test]
    fn css_plausible_input_never_panics(input in "[a-zA-Z#.:_ {}0-9%;,()\\-]+") {
        let _ = Stylesheet::parse(&input);
    }

    /// Empty string must not panic.
    #[test]
    fn css_empty_string_never_panics(_dummy in 0u8..=1u8) {
        let _ = Stylesheet::parse("");
    }

    /// Whitespace-only input must not panic.
    #[test]
    fn css_whitespace_never_panics(n in 0usize..=100usize) {
        let input = " ".repeat(n);
        let _ = Stylesheet::parse(&input);
    }
}
