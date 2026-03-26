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

/// Parser state extracted into a struct to work around rustc 1.94 ICE
/// in check_liveness when too many mutable locals exist in one function.
struct MdParseState {
    lines: Vec<RenderedLine>,
    current_text: String,
    current_style: Style,
    current_indent: u16,
    style_stack: Vec<Style>,
    list_stack: Vec<Option<u64>>,
    list_item_counter: Vec<u64>,
    in_code_block: bool,
    link_url: String,
}

impl MdParseState {
    fn new() -> Self {
        Self {
            lines: Vec::new(),
            current_text: String::new(),
            current_style: Style::default(),
            current_indent: 0,
            style_stack: vec![Style::default()],
            list_stack: Vec::new(),
            list_item_counter: Vec::new(),
            in_code_block: false,
            link_url: String::new(),
        }
    }

    fn flush_current(&mut self) {
        if !self.current_text.is_empty() {
            self.lines.push(RenderedLine {
                text: self.current_text.clone(),
                style: self.current_style,
                indent: self.current_indent,
            });
            self.current_text.clear();
        }
    }

    fn push_blank(&mut self) {
        self.lines.push(RenderedLine {
            text: String::new(),
            style: Style::default(),
            indent: 0,
        });
    }

    fn restore_style(&mut self) {
        self.current_style = *self.style_stack.last().unwrap_or(&Style::default());
    }

    fn process_event(&mut self, event: Event<'_>) {
        match event {
            Event::Start(Tag::Heading { level, .. }) => {
                self.current_style = match level {
                    HeadingLevel::H1 => Style::default()
                        .fg(Color::Rgb(0, 212, 255))
                        .add_modifier(Modifier::BOLD)
                        .add_modifier(Modifier::UNDERLINED),
                    HeadingLevel::H2 => Style::default()
                        .fg(Color::Rgb(0, 255, 163))
                        .add_modifier(Modifier::BOLD),
                    _ => Style::default()
                        .fg(Color::Rgb(200, 200, 220))
                        .add_modifier(Modifier::BOLD),
                };
            }
            Event::End(TagEnd::Heading(_)) => {
                self.flush_current();
                self.push_blank();
                self.restore_style();
            }

            Event::Start(Tag::Paragraph) => {}
            Event::End(TagEnd::Paragraph) => {
                self.flush_current();
                self.push_blank();
            }

            Event::Start(Tag::CodeBlock(_)) => {
                self.in_code_block = true;
                self.current_style = Style::default()
                    .fg(Color::Rgb(180, 180, 200))
                    .bg(Color::Rgb(20, 20, 30));
                self.current_indent = 2;
            }
            Event::End(TagEnd::CodeBlock) => {
                self.flush_current();
                self.in_code_block = false;
                self.current_indent = 0;
                self.restore_style();
                self.push_blank();
            }

            Event::Start(Tag::Emphasis) => {
                let new_style = self.current_style.add_modifier(Modifier::ITALIC);
                self.style_stack.push(self.current_style);
                self.current_style = new_style;
            }
            Event::End(TagEnd::Emphasis) => {
                self.current_style = self.style_stack.pop().unwrap_or_default();
            }

            Event::Start(Tag::Strong) => {
                let new_style = self.current_style.add_modifier(Modifier::BOLD);
                self.style_stack.push(self.current_style);
                self.current_style = new_style;
            }
            Event::End(TagEnd::Strong) => {
                self.current_style = self.style_stack.pop().unwrap_or_default();
            }

            Event::Start(Tag::Strikethrough) => {
                let new_style = self.current_style.add_modifier(Modifier::CROSSED_OUT);
                self.style_stack.push(self.current_style);
                self.current_style = new_style;
            }
            Event::End(TagEnd::Strikethrough) => {
                self.current_style = self.style_stack.pop().unwrap_or_default();
            }

            Event::Start(Tag::List(start_num)) => {
                self.list_stack.push(start_num);
                self.list_item_counter.push(start_num.unwrap_or(1));
            }
            Event::End(TagEnd::List(_)) => {
                self.list_stack.pop();
                self.list_item_counter.pop();
                if self.list_stack.is_empty() {
                    self.push_blank();
                }
            }
            Event::Start(Tag::Item) => {
                let prefix = if let Some(Some(_)) = self.list_stack.last() {
                    let n = *self.list_item_counter.last().unwrap_or(&1);
                    format!("  {}. ", n)
                } else {
                    "  • ".to_string()
                };
                self.current_text.push_str(&prefix);
                self.current_indent = 0;
            }
            Event::End(TagEnd::Item) => {
                self.flush_current();
                if let Some(counter) = self.list_item_counter.last_mut() {
                    *counter += 1;
                }
            }

            Event::Start(Tag::Link { dest_url, .. }) => {
                self.link_url = dest_url.to_string();
                self.style_stack.push(self.current_style);
                self.current_style = self.current_style
                    .fg(Color::Rgb(0, 178, 214))
                    .add_modifier(Modifier::UNDERLINED);
            }
            Event::End(TagEnd::Link) => {
                self.current_text.push_str(&format!(" [{}]", self.link_url));
                self.link_url.clear();
                self.current_style = self.style_stack.pop().unwrap_or_default();
            }

            Event::Start(Tag::Image { .. }) | Event::End(TagEnd::Image) => {}

            Event::Start(Tag::BlockQuote(_)) => {
                self.current_indent += 2;
            }
            Event::End(TagEnd::BlockQuote(_)) => {
                self.current_indent = self.current_indent.saturating_sub(2);
            }

            Event::Text(text) => {
                if self.in_code_block {
                    for line in text.lines() {
                        self.lines.push(RenderedLine {
                            text: line.to_string(),
                            style: self.current_style,
                            indent: self.current_indent,
                        });
                    }
                } else {
                    self.current_text.push_str(&text);
                }
            }

            Event::Code(code) => {
                self.current_text.push('`');
                self.current_text.push_str(&code);
                self.current_text.push('`');
            }

            Event::SoftBreak => {
                self.current_text.push(' ');
            }
            Event::HardBreak => {
                self.flush_current();
            }

            Event::Rule => {
                self.lines.push(RenderedLine {
                    text: "────────────────────────────────────────".to_string(),
                    style: Style::default().fg(Color::Rgb(74, 74, 90)),
                    indent: 0,
                });
            }

            Event::Html(_) | Event::InlineHtml(_) => {}
            Event::FootnoteReference(_) | Event::TaskListMarker(_) => {}
            _ => {}
        }
    }
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
/// - Links (`[text](url)`): rendered as "text \[url\]"
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
        let parser = Parser::new_ext(content, Options::empty());
        let mut state = MdParseState::new();

        for event in parser {
            state.process_event(event);
        }

        // Flush any remaining text
        if !state.current_text.is_empty() {
            state.lines.push(RenderedLine {
                text: state.current_text,
                style: state.current_style,
                indent: state.current_indent,
            });
        }

        state.lines
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
