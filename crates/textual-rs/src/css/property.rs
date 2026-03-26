use cssparser::{ParseError, Parser, Token};
use cssparser_color::{Color as ParsedColor};

use crate::css::types::{
    BorderStyle, Declaration, DockEdge, LayoutDirection, Overflow, Sides, TcssColor,
    TcssDimension, TcssDisplay, TcssValue, TextAlign, Visibility,
};

/// Error type for property parsing.
#[derive(Debug, Clone)]
pub enum PropertyParseError {
    UnknownProperty(String),
    InvalidValue(String),
}

/// Parse a color token sequence into TcssColor.
fn parse_color<'i>(
    input: &mut Parser<'i, '_>,
) -> Result<TcssColor, ParseError<'i, PropertyParseError>> {
    let location = input.current_source_location();
    let color = ParsedColor::parse(input).map_err(|e| {
        location.new_custom_error(PropertyParseError::InvalidValue(format!(
            "invalid color: {:?}",
            e
        )))
    })?;

    match color {
        ParsedColor::Rgba(rgba) => {
            if rgba.alpha >= 1.0 - f32::EPSILON {
                Ok(TcssColor::Rgb(rgba.red, rgba.green, rgba.blue))
            } else {
                // Convert 0.0-1.0 alpha to 0-255 range
                let alpha_u8 = (rgba.alpha * 255.0).round() as u8;
                Ok(TcssColor::Rgba(rgba.red, rgba.green, rgba.blue, alpha_u8))
            }
        }
        ParsedColor::CurrentColor => Ok(TcssColor::Reset),
        _ => Err(location.new_custom_error(PropertyParseError::InvalidValue(
            "unsupported color format".to_string(),
        ))),
    }
}

/// Parse a dimension value: number (Length), number% (Percent), number fr (Fraction), "auto" (Auto).
fn parse_dimension<'i>(
    input: &mut Parser<'i, '_>,
) -> Result<TcssDimension, ParseError<'i, PropertyParseError>> {
    let location = input.current_source_location();
    match input.next()? {
        Token::Ident(name) if name.eq_ignore_ascii_case("auto") => Ok(TcssDimension::Auto),
        Token::Number { value, .. } => Ok(TcssDimension::Length(*value)),
        Token::Percentage { unit_value, .. } => Ok(TcssDimension::Percent(*unit_value * 100.0)),
        Token::Dimension { value, unit, .. } if unit.eq_ignore_ascii_case("fr") => {
            Ok(TcssDimension::Fraction(*value))
        }
        other => Err(location.new_custom_error(PropertyParseError::InvalidValue(format!(
            "expected dimension value, got {:?}",
            other
        )))),
    }
}

/// Parse a non-negative float/number for padding/margin cell values.
fn parse_cells<'i>(
    input: &mut Parser<'i, '_>,
) -> Result<f32, ParseError<'i, PropertyParseError>> {
    let location = input.current_source_location();
    match input.next()? {
        Token::Number { value, .. } => Ok(*value),
        other => Err(location.new_custom_error(PropertyParseError::InvalidValue(format!(
            "expected number, got {:?}",
            other
        )))),
    }
}

/// Parse a declaration block (the part between `{` and `}`).
/// Returns a list of parsed declarations and skips unknown/invalid properties with collected errors.
pub fn parse_declaration_block<'i>(
    input: &mut Parser<'i, '_>,
) -> Result<Vec<Declaration>, ParseError<'i, PropertyParseError>> {
    let mut declarations = Vec::new();

    loop {
        input.skip_whitespace();
        if input.is_exhausted() {
            break;
        }

        // Parse property name
        let location = input.current_source_location();
        let property_name = match input.next() {
            Ok(Token::Ident(name)) => name.to_string(),
            Ok(_) | Err(_) => break,
        };

        input.skip_whitespace();

        // Expect colon
        match input.next() {
            Ok(Token::Colon) => {}
            _ => {
                // Skip to next semicolon and continue
                let _ = input.parse_until_after(cssparser::Delimiter::Semicolon, |_| {
                    Ok::<(), ParseError<'i, PropertyParseError>>(())
                });
                continue;
            }
        }

        input.skip_whitespace();

        // Parse value based on property name
        let result = parse_property_value(input, &property_name, location);

        match result {
            Ok(Some(value)) => {
                declarations.push(Declaration {
                    property: property_name,
                    value,
                });
            }
            Ok(None) | Err(_) => {
                // Unknown property or parse error — consume everything up to the
                // next semicolon so we don't eat the following property's tokens.
                let _ = input.parse_until_after(cssparser::Delimiter::Semicolon, |_| {
                    Ok::<(), ParseError<'i, PropertyParseError>>(())
                });
                continue;
            }
        }

        // Skip to semicolon or end
        input.skip_whitespace();
        let state = input.state();
        match input.next() {
            Ok(Token::Semicolon) => {}
            Ok(_) => {
                input.reset(&state);
            }
            Err(_) => break,
        }
    }

    Ok(declarations)
}

fn parse_property_value<'i>(
    input: &mut Parser<'i, '_>,
    property_name: &str,
    location: cssparser::SourceLocation,
) -> Result<Option<TcssValue>, ParseError<'i, PropertyParseError>> {
    match property_name {
        "color" | "background" => Ok(Some(TcssValue::Color(parse_color(input)?))),
        "border" => {
            let name = input
                .expect_ident_cloned()
                .map_err(|e| location.new_custom_error(PropertyParseError::InvalidValue(format!("{:?}", e))))?;
            let style = match name.as_ref() {
                "none" => BorderStyle::None,
                "solid" => BorderStyle::Solid,
                "rounded" => BorderStyle::Rounded,
                "heavy" => BorderStyle::Heavy,
                "double" => BorderStyle::Double,
                "ascii" => BorderStyle::Ascii,
                other => {
                    return Err(location.new_custom_error(PropertyParseError::InvalidValue(
                        format!("unknown border style: {}", other),
                    )));
                }
            };
            // Optionally consume a color after the border style (e.g. "border: solid #4a4a5a")
            // The color sets the widget's foreground color for the border.
            let color = parse_color(input).ok();
            if let Some(c) = color {
                return Ok(Some(TcssValue::BorderWithColor(style, c)));
            }
            Ok(Some(TcssValue::Border(style)))
        }
        "border-title" => {
            let s = input.expect_string_cloned().map_err(|e| {
                location.new_custom_error(PropertyParseError::InvalidValue(format!("{:?}", e)))
            })?;
            Ok(Some(TcssValue::String(s.to_string())))
        }
        "width" | "height" | "min-width" | "min-height" | "max-width" | "max-height" => {
            Ok(Some(TcssValue::Dimension(parse_dimension(input)?)))
        }
        "padding" | "margin" => {
            // Parse 1-4 values
            let first = parse_cells(input)?;
            input.skip_whitespace();

            // Try to peek for more values
            let state = input.state();
            match input.next_including_whitespace() {
                Ok(Token::Number { value, .. }) => {
                    let second = *value;
                    input.skip_whitespace();
                    let state2 = input.state();
                    match input.next_including_whitespace() {
                        Ok(Token::Number { value, .. }) => {
                            let third = *value;
                            input.skip_whitespace();
                            let state3 = input.state();
                            match input.next_including_whitespace() {
                                Ok(Token::Number { value, .. }) => {
                                    let fourth = *value;
                                    // 4 values: top right bottom left
                                    Ok(Some(TcssValue::Sides(Sides {
                                        top: first,
                                        right: second,
                                        bottom: third,
                                        left: fourth,
                                    })))
                                }
                                _ => {
                                    input.reset(&state3);
                                    // 3 values: top, left/right, bottom
                                    Ok(Some(TcssValue::Sides(Sides {
                                        top: first,
                                        right: second,
                                        bottom: third,
                                        left: second,
                                    })))
                                }
                            }
                        }
                        _ => {
                            input.reset(&state2);
                            // 2 values: top/bottom + left/right
                            Ok(Some(TcssValue::Sides(Sides {
                                top: first,
                                right: second,
                                bottom: first,
                                left: second,
                            })))
                        }
                    }
                }
                _ => {
                    input.reset(&state);
                    // 1 value: all sides
                    Ok(Some(TcssValue::Float(first)))
                }
            }
        }
        "display" => {
            let name = input
                .expect_ident_cloned()
                .map_err(|e| location.new_custom_error(PropertyParseError::InvalidValue(format!("{:?}", e))))?;
            let d = match name.as_ref() {
                "flex" => TcssDisplay::Flex,
                "grid" => TcssDisplay::Grid,
                "block" => TcssDisplay::Block,
                "none" => TcssDisplay::None,
                other => {
                    return Err(location.new_custom_error(PropertyParseError::InvalidValue(
                        format!("unknown display value: {}", other),
                    )));
                }
            };
            Ok(Some(TcssValue::Display(d)))
        }
        "visibility" => {
            let name = input
                .expect_ident_cloned()
                .map_err(|e| location.new_custom_error(PropertyParseError::InvalidValue(format!("{:?}", e))))?;
            let v = match name.as_ref() {
                "visible" => Visibility::Visible,
                "hidden" => Visibility::Hidden,
                other => {
                    return Err(location.new_custom_error(PropertyParseError::InvalidValue(
                        format!("unknown visibility value: {}", other),
                    )));
                }
            };
            Ok(Some(TcssValue::Visibility(v)))
        }
        "opacity" | "flex-grow" => {
            let v = input.expect_number().map_err(|e| {
                location.new_custom_error(PropertyParseError::InvalidValue(format!("{:?}", e)))
            })?;
            Ok(Some(TcssValue::Float(v)))
        }
        "text-align" => {
            let name = input
                .expect_ident_cloned()
                .map_err(|e| location.new_custom_error(PropertyParseError::InvalidValue(format!("{:?}", e))))?;
            let a = match name.as_ref() {
                "left" => TextAlign::Left,
                "center" => TextAlign::Center,
                "right" => TextAlign::Right,
                other => {
                    return Err(location.new_custom_error(PropertyParseError::InvalidValue(
                        format!("unknown text-align value: {}", other),
                    )));
                }
            };
            Ok(Some(TcssValue::TextAlign(a)))
        }
        "overflow" => {
            let name = input
                .expect_ident_cloned()
                .map_err(|e| location.new_custom_error(PropertyParseError::InvalidValue(format!("{:?}", e))))?;
            let o = match name.as_ref() {
                "visible" => Overflow::Visible,
                "hidden" => Overflow::Hidden,
                "scroll" => Overflow::Scroll,
                "auto" => Overflow::Auto,
                other => {
                    return Err(location.new_custom_error(PropertyParseError::InvalidValue(
                        format!("unknown overflow value: {}", other),
                    )));
                }
            };
            Ok(Some(TcssValue::Overflow(o)))
        }
        "scrollbar-gutter" => {
            let name = input
                .expect_ident_cloned()
                .map_err(|e| location.new_custom_error(PropertyParseError::InvalidValue(format!("{:?}", e))))?;
            let b = match name.as_ref() {
                "stable" => true,
                "auto" => false,
                other => {
                    return Err(location.new_custom_error(PropertyParseError::InvalidValue(
                        format!("unknown scrollbar-gutter value: {}", other),
                    )));
                }
            };
            Ok(Some(TcssValue::Bool(b)))
        }
        "dock" => {
            let name = input
                .expect_ident_cloned()
                .map_err(|e| location.new_custom_error(PropertyParseError::InvalidValue(format!("{:?}", e))))?;
            let d = match name.as_ref() {
                "top" => DockEdge::Top,
                "bottom" => DockEdge::Bottom,
                "left" => DockEdge::Left,
                "right" => DockEdge::Right,
                other => {
                    return Err(location.new_custom_error(PropertyParseError::InvalidValue(
                        format!("unknown dock value: {}", other),
                    )));
                }
            };
            Ok(Some(TcssValue::DockEdge(d)))
        }
        "grid-template-columns" | "grid-template-rows" => {
            // Parse space-separated list of dimensions
            let mut dims = Vec::new();
            loop {
                input.skip_whitespace();
                let state = input.state();
                match parse_dimension(input) {
                    Ok(d) => dims.push(d),
                    Err(_) => {
                        input.reset(&state);
                        break;
                    }
                }
            }
            if dims.is_empty() {
                Ok(None)
            } else {
                Ok(Some(TcssValue::Dimensions(dims)))
            }
        }
        "layout-direction" => {
            let name = input
                .expect_ident_cloned()
                .map_err(|e| location.new_custom_error(PropertyParseError::InvalidValue(format!("{:?}", e))))?;
            let d = match name.as_ref() {
                "vertical" => LayoutDirection::Vertical,
                "horizontal" => LayoutDirection::Horizontal,
                other => {
                    return Err(location.new_custom_error(PropertyParseError::InvalidValue(
                        format!("unknown layout-direction value: {}", other),
                    )));
                }
            };
            Ok(Some(TcssValue::LayoutDirection(d)))
        }
        // Unknown property — skip
        _other => Ok(None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_decl(css: &str) -> Declaration {
        let input_str = format!("{};", css);
        let mut input = cssparser::ParserInput::new(&input_str);
        let mut parser = cssparser::Parser::new(&mut input);
        let decls = parse_declaration_block(&mut parser).expect("parse failed");
        assert!(!decls.is_empty(), "no declaration parsed from: {}", css);
        decls.into_iter().next().unwrap()
    }

    fn parse_decl_value(css: &str) -> TcssValue {
        parse_decl(css).value
    }

    #[test]
    fn parse_color_named() {
        let val = parse_decl_value("color: red");
        // "red" -> Rgb(255, 0, 0)
        assert!(matches!(val, TcssValue::Color(TcssColor::Rgb(255, 0, 0))));
    }

    #[test]
    fn parse_color_hex_6() {
        let val = parse_decl_value("color: #ff0000");
        assert!(matches!(val, TcssValue::Color(TcssColor::Rgb(255, 0, 0))));
    }

    #[test]
    fn parse_color_rgb_function() {
        let val = parse_decl_value("color: rgb(255, 0, 0)");
        assert!(matches!(val, TcssValue::Color(TcssColor::Rgb(255, 0, 0))));
    }

    #[test]
    fn parse_color_rgba_function() {
        // CSS rgba uses 0-1 alpha; 0.5 = ~50% opacity, stored as alpha_u8 ~128
        let val = parse_decl_value("color: rgba(255, 0, 0, 0.5)");
        assert!(matches!(val, TcssValue::Color(TcssColor::Rgba(255, 0, 0, _))));
    }

    #[test]
    fn parse_color_hex_3() {
        let val = parse_decl_value("color: #f00");
        assert!(matches!(val, TcssValue::Color(TcssColor::Rgb(255, 0, 0))));
    }

    #[test]
    fn parse_width_number() {
        let val = parse_decl_value("width: 20");
        assert_eq!(val, TcssValue::Dimension(TcssDimension::Length(20.0)));
    }

    #[test]
    fn parse_width_percent() {
        let val = parse_decl_value("width: 50%");
        assert_eq!(val, TcssValue::Dimension(TcssDimension::Percent(50.0)));
    }

    #[test]
    fn parse_width_fraction() {
        let val = parse_decl_value("width: 1fr");
        assert_eq!(val, TcssValue::Dimension(TcssDimension::Fraction(1.0)));
    }

    #[test]
    fn parse_width_auto() {
        let val = parse_decl_value("width: auto");
        assert_eq!(val, TcssValue::Dimension(TcssDimension::Auto));
    }

    #[test]
    fn parse_border_solid() {
        let val = parse_decl_value("border: solid");
        assert_eq!(val, TcssValue::Border(BorderStyle::Solid));
    }

    #[test]
    fn parse_border_rounded() {
        let val = parse_decl_value("border: rounded");
        assert_eq!(val, TcssValue::Border(BorderStyle::Rounded));
    }

    #[test]
    fn parse_display_none() {
        let val = parse_decl_value("display: none");
        assert_eq!(val, TcssValue::Display(TcssDisplay::None));
    }

    #[test]
    fn parse_opacity() {
        let val = parse_decl_value("opacity: 0.5");
        assert_eq!(val, TcssValue::Float(0.5));
    }

    #[test]
    fn parse_dock_top() {
        let val = parse_decl_value("dock: top");
        assert_eq!(val, TcssValue::DockEdge(DockEdge::Top));
    }
}
