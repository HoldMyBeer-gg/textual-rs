/// textual-rs demo — widget showcase with a tabbed layout.
///
/// Demonstrates all major widget categories in a lazeport-inspired dark theme.
///
/// Layout:
/// ┌──────────────── Header (textual-rs demo) ──────────────────┐
/// ├────────────────────────────────────────────────────────────┤
/// │  Inputs | Display | Layout | Interactive   (TabbedContent) │
/// │ ┌──────────────────────────────────────────────────────── ┐│
/// │ │  Active pane content                                    ││
/// │ └──────────────────────────────────────────────────────── ┘│
/// ├────────────────────────────────────────────────────────────┤
/// │  Footer (ctrl+p Command Palette  q Quit)                   │
/// └────────────────────────────────────────────────────────────┘
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use textual_rs::{
    App, Widget,
    Button, ButtonVariant,
    Checkbox, Switch,
    RadioSet,
    Input,
    Label,
    DataTable, ColumnDef,
    ProgressBar, Sparkline,
    ListView,
    TabbedContent,
    Header, Footer,
    Collapsible, Placeholder,
    Markdown,
    Tree, TreeNode,
    Select,
    Horizontal, Vertical,
};
use textual_rs::widget::context::AppContext;

// ---- Inputs tab ----

/// Inputs tab: form controls — Input, RadioSet, Checkbox, Switch, Button.
struct InputsPane;

impl Widget for InputsPane {
    fn widget_type_name(&self) -> &'static str {
        "InputsPane"
    }

    fn compose(&self) -> Vec<Box<dyn Widget>> {
        vec![
            Box::new(Label::new("Form Controls")),
            Box::new(Input::new("Type something...")),
            Box::new(Input::new("email@example.com").with_validator(|v| v.contains('@'))),
            Box::new(Checkbox::new("Enable notifications", true)),
            Box::new(Switch::new(false)),
            Box::new(RadioSet::new(vec![
                "Option A".to_string(),
                "Option B".to_string(),
                "Option C".to_string(),
            ])),
            Box::new(Button::new("Submit").with_variant(ButtonVariant::Primary)),
            Box::new(Button::new("Cancel")),
        ]
    }

    fn render(&self, _ctx: &AppContext, _area: Rect, _buf: &mut Buffer) {}
}

// ---- Display tab ----

/// Display tab: Label, ProgressBar, Sparkline, Markdown, Placeholder.
struct DisplayPane;

impl Widget for DisplayPane {
    fn widget_type_name(&self) -> &'static str {
        "DisplayPane"
    }

    fn compose(&self) -> Vec<Box<dyn Widget>> {
        vec![
            Box::new(Label::new("Static text via Label widget")),
            Box::new(Label::new("Build progress: 65%")),
            Box::new(ProgressBar::new(0.65)),
            Box::new(Label::new("CPU activity:")),
            Box::new(Sparkline::new(vec![1.0, 3.0, 7.0, 2.0, 8.0, 4.0, 6.0, 5.0, 9.0, 3.0])),
            Box::new(Markdown::new("# textual-rs\n\nA **Rust** TUI framework inspired by Textual.\n\n- 22 built-in widgets\n- Reactive state\n- CSS styling")),
            Box::new(Placeholder::new()),
        ]
    }

    fn render(&self, _ctx: &AppContext, _area: Rect, _buf: &mut Buffer) {}
}

// ---- Layout tab ----

/// Layout tab: Horizontal/Vertical containers, Collapsible section.
struct LayoutPane;

impl Widget for LayoutPane {
    fn widget_type_name(&self) -> &'static str {
        "LayoutPane"
    }

    fn compose(&self) -> Vec<Box<dyn Widget>> {
        vec![
            Box::new(Label::new("Horizontal container (3 panels):")),
            Box::new(Horizontal::with_children(vec![
                Box::new(Label::new("Panel 1")),
                Box::new(Label::new("Panel 2")),
                Box::new(Label::new("Panel 3")),
            ])),
            Box::new(Label::new("Vertical container (2 rows):")),
            Box::new(Vertical::with_children(vec![
                Box::new(Label::new("Row A")),
                Box::new(Label::new("Row B")),
            ])),
            Box::new(Collapsible::new("Advanced Options", vec![
                Box::new(Label::new("Option 1: enabled")),
                Box::new(Label::new("Option 2: disabled")),
            ])),
        ]
    }

    fn render(&self, _ctx: &AppContext, _area: Rect, _buf: &mut Buffer) {}
}

// ---- Interactive tab ----

/// Interactive tab: Button, ListView, DataTable, Tree, Select.
struct InteractivePane;

impl Widget for InteractivePane {
    fn widget_type_name(&self) -> &'static str {
        "InteractivePane"
    }

    fn compose(&self) -> Vec<Box<dyn Widget>> {
        let mut table = DataTable::new(vec![
            ColumnDef::new("Name").with_width(18),
            ColumnDef::new("Type").with_width(14),
            ColumnDef::new("Status").with_width(10),
        ]);
        table.add_row(vec!["Label".into(), "Display".into(), "Stable".into()]);
        table.add_row(vec!["Button".into(), "Interactive".into(), "Stable".into()]);
        table.add_row(vec!["Input".into(), "Form".into(), "Stable".into()]);
        table.add_row(vec!["ListView".into(), "List".into(), "Stable".into()]);
        table.add_row(vec!["DataTable".into(), "Table".into(), "Stable".into()]);

        // Tree with a sample directory hierarchy
        let widget_dir = TreeNode::with_children("widget/", vec![
            TreeNode::new("mod.rs"),
            TreeNode::new("button.rs"),
            TreeNode::new("input.rs"),
        ]);
        let root = TreeNode::with_children("src/", vec![
            TreeNode::new("app.rs"),
            TreeNode::new("lib.rs"),
            widget_dir,
        ]);
        let tree = Tree::new(root);

        let select = Select::new(vec![
            "Rust".to_string(),
            "Python".to_string(),
            "Go".to_string(),
            "TypeScript".to_string(),
            "Zig".to_string(),
        ]);

        vec![
            Box::new(Button::new("Click Me").with_variant(ButtonVariant::Primary)),
            Box::new(ListView::new(vec![
                "#general".to_string(),
                "#rust".to_string(),
                "#tui-dev".to_string(),
                "#announcements".to_string(),
                "#help".to_string(),
                "#off-topic".to_string(),
                "#code-review".to_string(),
                "#random".to_string(),
                "#feedback".to_string(),
                "#releases".to_string(),
            ])),
            Box::new(table),
            Box::new(tree),
            Box::new(select),
        ]
    }

    fn render(&self, _ctx: &AppContext, _area: Rect, _buf: &mut Buffer) {}
}

// ---- Top-level screen ----

/// Root screen: Header + TabbedContent (4 tabs) + Footer.
struct DemoScreen;

impl Widget for DemoScreen {
    fn widget_type_name(&self) -> &'static str {
        "DemoScreen"
    }

    fn compose(&self) -> Vec<Box<dyn Widget>> {
        let tabbed = TabbedContent::new(
            vec![
                "Inputs".to_string(),
                "Display".to_string(),
                "Layout".to_string(),
                "Interactive".to_string(),
            ],
            vec![
                Box::new(InputsPane),
                Box::new(DisplayPane),
                Box::new(LayoutPane),
                Box::new(InteractivePane),
            ],
        );

        vec![
            Box::new(Header::new("textual-rs demo").with_subtitle("A showcase of built-in widgets")),
            Box::new(tabbed),
            Box::new(Footer),
        ]
    }

    fn render(&self, _ctx: &AppContext, _area: Rect, _buf: &mut Buffer) {}
}

fn main() -> anyhow::Result<()> {
    let mut app = App::new(|| Box::new(DemoScreen))
        .with_css(include_str!("demo.tcss"));
    app.run()
}
