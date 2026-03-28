//! TCSS stylesheet parser built on cssparser.

use cssparser::{
    AtRuleParser, ParseError, Parser, ParserInput, QualifiedRuleParser, StyleSheetParser,
};

use crate::css::property::{parse_declaration_block, PropertyParseError};
use crate::css::selector::{Selector, SelectorParseError, SelectorParser};
use crate::css::types::Declaration;

/// A parsed TCSS rule: a selector list + declaration block.
#[derive(Debug, Clone)]
pub struct Rule {
    /// The list of selectors this rule applies to.
    pub selectors: Vec<Selector>,
    /// The property declarations to apply when the selectors match.
    pub declarations: Vec<Declaration>,
}

/// Custom parse error for TCSS.
#[derive(Debug, Clone)]
pub enum TcssParseError {
    /// The selector string could not be parsed.
    InvalidSelector(String),
    /// An unrecognized CSS property was encountered.
    InvalidProperty(String),
    /// A property value could not be parsed.
    InvalidValue(String),
}

impl From<SelectorParseError> for TcssParseError {
    fn from(e: SelectorParseError) -> Self {
        TcssParseError::InvalidSelector(e.0)
    }
}

impl From<PropertyParseError> for TcssParseError {
    fn from(e: PropertyParseError) -> Self {
        match e {
            PropertyParseError::UnknownProperty(s) => TcssParseError::InvalidProperty(s),
            PropertyParseError::InvalidValue(s) => TcssParseError::InvalidValue(s),
        }
    }
}

/// Rule parser for the TCSS stylesheet parser.
pub struct TcssRuleParser;

impl<'i> AtRuleParser<'i> for TcssRuleParser {
    type Prelude = ();
    type AtRule = Rule;
    type Error = TcssParseError;
}

impl<'i> QualifiedRuleParser<'i> for TcssRuleParser {
    type Prelude = Vec<Selector>;
    type QualifiedRule = Rule;
    type Error = TcssParseError;

    fn parse_prelude<'t>(
        &mut self,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self::Prelude, ParseError<'i, Self::Error>> {
        SelectorParser::parse_selector_list(input).map_err(|e| e.into::<TcssParseError>())
    }

    fn parse_block<'t>(
        &mut self,
        prelude: Self::Prelude,
        _start: &cssparser::ParserState,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self::QualifiedRule, ParseError<'i, Self::Error>> {
        let declarations =
            parse_declaration_block(input).map_err(|e| e.into::<TcssParseError>())?;
        Ok(Rule {
            selectors: prelude,
            declarations,
        })
    }
}

/// Parse a TCSS stylesheet string into rules and error messages.
///
/// Returns `(rules, errors)` where errors contain line-number information.
pub fn parse_stylesheet(css: &str) -> (Vec<Rule>, Vec<String>) {
    let mut input = ParserInput::new(css);
    let mut parser = Parser::new(&mut input);
    let mut rule_parser = TcssRuleParser;

    let mut rules = Vec::new();
    let mut errors = Vec::new();

    let sheet_parser = StyleSheetParser::new(&mut parser, &mut rule_parser);
    for result in sheet_parser {
        match result {
            Ok(rule) => rules.push(rule),
            Err((parse_error, slice)) => {
                let loc = parse_error.location;
                // loc.line is 0-indexed; add 1 for human-readable line numbers
                let msg = format!(
                    "CSS parse error at line {}, column {}: {:?} (near {:?})",
                    loc.line + 1,
                    loc.column,
                    parse_error.kind,
                    slice
                );
                errors.push(msg);
            }
        }
    }

    (rules, errors)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::css::selector::Selector;
    use crate::css::types::{TcssColor, TcssValue};

    #[test]
    fn parse_stylesheet_single_rule() {
        let (rules, errors) = parse_stylesheet("Button { color: red; }");
        assert!(errors.is_empty(), "unexpected errors: {:?}", errors);
        assert_eq!(rules.len(), 1);
        assert_eq!(
            rules[0].selectors,
            vec![Selector::Type("Button".to_string())]
        );
        assert_eq!(rules[0].declarations.len(), 1);
        assert_eq!(rules[0].declarations[0].property, "color");
        assert!(matches!(
            rules[0].declarations[0].value,
            TcssValue::Color(TcssColor::Rgb(255, 0, 0))
        ));
    }

    #[test]
    fn parse_stylesheet_three_rules() {
        let css = r#"
            Button { color: red; }
            .active { display: block; }
            #sidebar { width: 20; }
        "#;
        let (rules, errors) = parse_stylesheet(css);
        assert!(errors.is_empty(), "unexpected errors: {:?}", errors);
        assert_eq!(rules.len(), 3);
    }

    #[test]
    fn parse_stylesheet_syntax_error_collects_line_number() {
        // Second rule has a syntax error (invalid selector with $)
        let css = r#"Button { color: red; }
$invalid { color: blue; }
Label { display: flex; }"#;
        let (rules, errors) = parse_stylesheet(css);
        // Valid rules should still be parsed
        assert!(!errors.is_empty(), "expected at least one error");
        // Error should contain line info
        assert!(
            errors[0].contains("line 2") || errors[0].contains("2"),
            "error should mention line number: {}",
            errors[0]
        );
        // The valid rules before/after the error should still be parsed
        assert!(
            rules.len() >= 1,
            "should have parsed at least one valid rule"
        );
    }

    #[test]
    fn parse_stylesheet_empty_returns_empty() {
        let (rules, errors) = parse_stylesheet("");
        assert!(rules.is_empty());
        assert!(errors.is_empty());
    }

    #[test]
    fn parse_stylesheet_multiple_declarations() {
        let css = "Button { color: red; display: block; opacity: 0.5; }";
        let (rules, errors) = parse_stylesheet(css);
        assert!(errors.is_empty(), "unexpected errors: {:?}", errors);
        assert_eq!(rules.len(), 1);
        assert_eq!(rules[0].declarations.len(), 3);
    }
}
