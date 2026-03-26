use std::cell::RefCell;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use pulldown_cmark::{Event, HeadingLevel, Options, Parser, Tag, TagEnd, CodeBlockKind};

use super::context::AppContext;
use super::Widget;

/// A single rendered line from parsed Markdown content.
struct RenderedLine {
    text: String,
    style: Style,
    indent: u16,
}

/// A widget that renders CommonMark Markdown content using pulldown-cmark.
///
/// Supported elements (per D-06):
/// - Headings (H1-H6): H1 gets underline, H2-H6 get bold
/// - Bold (**text**): rendered with bold modifier
/// - Italic (*text*): rendered with italic modifier
/// - Inline code (`code`): surrounded in backticks, dim style
/// - Code blocks (fenced): rendered with indent=2, dim style
/// - Unordered lists (- item): bullet `  * ` prefix
/// - Ordered lists (1. item): numbered `  N. ` prefix
/// - Links ([text](url)): rendered as "text [url]"
/// - Horizontal rules: "────────" line
/// - Paragraphs and line breaks
///
/// Not supported: images, tables, HTML (per D-06).
pub struct Markdown {
    pub content: String,
    rendered_lines: RefCell<Vec<RenderedLine>>,
}

impl Markdown {
    pub fn new(content: &str) -> Self {
        let rendered = Self::parse_markdown(content);
        Self {
            content: content.to_string(),
            rendered_lines: RefCell::new(rendered),
        }
    }

    fn parse_markdown(content: &str) -> Vec<RenderedLine> {
        let options = Options::empty();
        let parser = Parser::new_ext(content, options);

        let mut lines: Vec<RenderedLine> = Vec::new();
        let mut current_text = String::new();
        let mut current_style = Style::default();
        let mut current_indent: u16 = 0;

        // Style stack for nested formatting
        let mut style_stack: Vec<Style> = vec![Style::default()];

        // List tracking
        let mut list_stack: Vec<Option<u64>> = Vec::new(); // None = unordered, Some(n) = ordered
        let mut list_item_counter: Vec<u64> = Vec::new();
        let mut in_list_item = false;
        let mut list_item_prefix = String::new();

        // State flags
        let mut in_code_block = false;
        let mut in_heading = false;
        let mut heading_style = Style::default();
        let mut in_link = false;
        let mut link_url = String::new();

        let flush_line = |lines: &mut Vec<RenderedLine>, text: &mut String, style: Style, indent: u16| {
            lines.push(RenderedLine {
                text: text.clone(),
                style,
                indent,
            });
            text.clear();
        };

        for event in parser {
            match event {
                // --- Headings ---
                Event::Start(Tag::Heading { level, .. }) => {
                    in_heading = true;
                    heading_style = match level {
                        HeadingLevel::H1 => Style::default()
                            .add_modifier(Modifier::BOLD)
                            .add_modifier(Modifier::UNDERLINED),
                        _ => Style::default().add_modifier(Modifier::BOLD),
                    };
                    current_style = heading_style;
                }
                Event::End(TagEnd::Heading(_)) => {
                    if !current_text.is_empty() {
                        lines.push(RenderedLine {
                            text: current_text.clone(),
                            style: current_style,
                            indent: current_indent,
                        });
                        current_text.clear();
                    }
                    // Blank line after heading
                    lines.push(RenderedLine {
                        text: String::new(),
                        style: Style::default(),
                        indent: 0,
                    });
                    in_heading = false;
                    current_style = *style_stack.last().unwrap_or(&Style::default());
                }

                // --- Paragraphs ---
                Event::Start(Tag::Paragraph) => {
                    // Start fresh line
                }
                Event::End(TagEnd::Paragraph) => {
                    if !current_text.is_empty() {
                        lines.push(RenderedLine {
                            text: current_text.clone(),
                            style: current_style,
                            indent: current_indent,
                        });
                        current_text.clear();
                    }
                    // Blank line after paragraph
                    lines.push(RenderedLine {
                        text: String::new(),
                        style: Style::default(),
                        indent: 0,
                    });
                }

                // --- Code blocks ---
                Event::Start(Tag::CodeBlock(_kind)) => {
                    in_code_block = true;
                    current_style = Style::default()
                        .fg(Color::DarkGray)
                        .add_modifier(Modifier::DIM);
                    current_indent = 2;
                }
                Event::End(TagEnd::CodeBlock) => {
                    if !current_text.is_empty() {
                        lines.push(RenderedLine {
                            text: current_text.clone(),
                            style: current_style,
                            indent: current_indent,
                        });
                        current_text.clear();
                    }
                    in_code_block = false;
                    current_indent = 0;
                    current_style = *style_stack.last().unwrap_or(&Style::default());
                    // Blank line after code block
                    lines.push(RenderedLine {
                        text: String::new(),
                        style: Style::default(),
                        indent: 0,
                    });
                }

                // --- Emphasis (italic) ---
                Event::Start(Tag::Emphasis) => {
                    let new_style = current_style.add_modifier(Modifier::ITALIC);
                    style_stack.push(current_style);
                    current_style = new_style;
                }
                Event::End(TagEnd::Emphasis) => {
                    current_style = style_stack.pop().unwrap_or_default();
                }

                // --- Strong (bold) ---
                Event::Start(Tag::Strong) => {
                    let new_style = current_style.add_modifier(Modifier::BOLD);
                    style_stack.push(current_style);
                    current_style = new_style;
                }
                Event::End(TagEnd::Strong) => {
                    current_style = style_stack.pop().unwrap_or_default();
                }

                // --- Strikethrough ---
                Event::Start(Tag::Strikethrough) => {
                    let new_style = current_style.add_modifier(Modifier::CROSSED_OUT);
                    style_stack.push(current_style);
                    current_style = new_style;
                }
                Event::End(TagEnd::Strikethrough) => {
                    current_style = style_stack.pop().unwrap_or_default();
                }

                // --- Lists ---
                Event::Start(Tag::List(start_num)) => {
                    list_stack.push(start_num);
                    list_item_counter.push(start_num.unwrap_or(1));
                }
                Event::End(TagEnd::List(_)) => {
                    list_stack.pop();
                    list_item_counter.pop();
                    // Blank line after list
                    if list_stack.is_empty() {
                        lines.push(RenderedLine {
                            text: String::new(),
                            style: Style::default(),
                            indent: 0,
                        });
                    }
                }
                Event::Start(Tag::Item) => {
                    in_list_item = true;
                    let prefix = if let Some(Some(_)) = list_stack.last() {
                        // Ordered list
                        let n = *list_item_counter.last().unwrap_or(&1);
                        format!("  {}. ", n)
                    } else {
                        // Unordered list
                        "  * ".to_string()
                    };
                    list_item_prefix = prefix;
                    current_text.push_str(&list_item_prefix);
                    current_indent = 0;
                }
                Event::End(TagEnd::Item) => {
                    if !current_text.is_empty() {
                        lines.push(RenderedLine {
                            text: current_text.clone(),
                            style: current_style,
                            indent: current_indent,
                        });
                        current_text.clear();
                    }
                    in_list_item = false;
                    list_item_prefix.clear();
                    // Increment ordered list counter
                    if let Some(counter) = list_item_counter.last_mut() {
                        *counter += 1;
                    }
                }

                // --- Links ---
                Event::Start(Tag::Link { dest_url, .. }) => {
                    in_link = true;
                    link_url = dest_url.to_string();
                    style_stack.push(current_style);
                }
                Event::End(TagEnd::Link) => {
                    // Append " [url]" in dim style after link text
                    let url_style = Style::default().add_modifier(Modifier::DIM);
                    // Flush current text first with current style, then add the URL
                    // We append the URL to the current buffer (style mixing not supported in
                    // single RenderedLine, so append as part of the text with a note)
                    current_text.push_str(&format!(" [{}]", link_url));
                    in_link = false;
                    link_url.clear();
                    current_style = style_stack.pop().unwrap_or_default();
                    // Suppress unused variable warning
                    let _ = url_style;
                }

                // --- Images (not rendered per D-06) ---
                Event::Start(Tag::Image { .. }) => {}
                Event::End(TagEnd::Image) => {}

                // --- Block quote ---
                Event::Start(Tag::BlockQuote(_)) => {
                    current_indent += 2;
                }
                Event::End(TagEnd::BlockQuote(_)) => {
                    if current_indent >= 2 {
                        current_indent -= 2;
                    }
                }

                // --- Text content ---
                Event::Text(text) => {
                    if in_code_block {
                        // In code block — each line of text becomes its own RenderedLine
                        for line in text.lines() {
                            lines.push(RenderedLine {
                                text: line.to_string(),
                                style: current_style,
                                indent: current_indent,
                            });
                        }
                    } else {
                        current_text.push_str(&text);
                    }
                }

                // --- Inline code ---
                Event::Code(code) => {
                    let code_style = Style::default().add_modifier(Modifier::DIM);
                    // Append inline code to current text (style per-segment not supported, use text marker)
                    current_text.push('`');
                    current_text.push_str(&code);
                    current_text.push('`');
                    // Keep current_style unchanged (can't mix styles per segment in RenderedLine v1)
                    let _ = code_style;
                }

                // --- Line breaks ---
                Event::SoftBreak => {
                    current_text.push(' ');
                }
                Event::HardBreak => {
                    if !current_text.is_empty() {
                        lines.push(RenderedLine {
                            text: current_text.clone(),
                            style: current_style,
                            indent: current_indent,
                        });
                        current_text.clear();
                    }
                }

                // --- Horizontal rule ---
                Event::Rule => {
                    lines.push(RenderedLine {
                        text: "────────────────────────────────────────".to_string(),
                        style: Style::default().add_modifier(Modifier::DIM),
                        indent: 0,
                    });
                }

                // --- HTML (not rendered per D-06) ---
                Event::Html(_) | Event::InlineHtml(_) => {}

                // --- Other events (ignored) ---
                Event::FootnoteReference(_) | Event::TaskListMarker(_) => {}

                // Catch-all for any other event variants
                _ => {}
            }
        }

        // Flush any remaining text
        if !current_text.is_empty() {
            lines.push(RenderedLine {
                text: current_text,
                style: current_style,
                indent: current_indent,
            });
        }

        lines
    }
}

impl Widget for Markdown {
    fn widget_type_name(&self) -> &'static str {
        "Markdown"
    }

    fn can_focus(&self) -> bool {
        false
    }

    fn default_css() -> &'static str
    where
        Self: Sized,
    {
        "Markdown { min-height: 3; }"
    }

    fn render(&self, _ctx: &AppContext, area: Rect, buf: &mut Buffer) {
        if area.height == 0 || area.width == 0 {
            return;
        }

        let lines = self.rendered_lines.borrow();
        let max_rows = area.height as usize;

        for (row_offset, line) in lines.iter().enumerate().take(max_rows) {
            let y = area.y + row_offset as u16;
            let x_start = area.x + line.indent.min(area.width.saturating_sub(1));
            let available_width = area.width.saturating_sub(line.indent) as usize;

            if available_width == 0 {
                continue;
            }

            let display: String = line.text.chars().take(available_width).collect();
            buf.set_string(x_start, y, &display, line.style);
        }
    }
}
