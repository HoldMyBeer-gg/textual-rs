/// Widget showcase demo — tabbed layout demonstrating textual-rs built-in widgets.
///
/// Layout:
/// ┌──────────────────── Header (dock: top) ─────────────────────┐
/// ├─────────────────────────────────────────────────────────────┤
/// │  Controls | Data | Lists    (TabbedContent)                 │
/// │ ┌─────────────────────────────────────────────────────────┐ │
/// │ │  Active pane content (compose-based children)           │ │
/// │ └─────────────────────────────────────────────────────────┘ │
/// ├─────────────────────────────────────────────────────────────┤
/// │  Footer (dock: bottom)                                      │
/// └─────────────────────────────────────────────────────────────┘
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
    ListView, Log,
    TabbedContent,
    Header, Footer,
};
use textual_rs::widget::context::AppContext;

// ---- Demo stylesheet ----

const DEMO_CSS: &str = r#"
DemoScreen {
    layout-direction: vertical;
    background: rgb(10,10,15);
    color: rgb(224,224,224);
}
Header {
    height: 1;
    background: rgb(18,18,26);
    color: rgb(0,212,255);
}
Footer {
    height: 1;
    background: rgb(18,18,26);
    color: rgb(224,224,224);
}
TabbedContent {
    flex-grow: 1;
}
ControlsPane {
    layout-direction: vertical;
    padding: 1;
}
DataPane {
    layout-direction: vertical;
    padding: 1;
}
ListsPane {
    layout-direction: horizontal;
}
Button {
    border: heavy;
    min-width: 16;
    height: 3;
    color: rgb(0,255,163);
}
Input {
    border: rounded;
    height: 3;
    color: rgb(224,224,224);
}
DataTable {
    border: rounded;
    min-height: 8;
    color: rgb(224,224,224);
}
ListView {
    border: rounded;
    min-height: 6;
    flex-grow: 1;
    color: rgb(224,224,224);
}
Log {
    border: rounded;
    min-height: 6;
    flex-grow: 1;
    color: rgb(0,255,163);
}
ProgressBar {
    height: 1;
    color: rgb(0,255,163);
}
Sparkline {
    height: 1;
    color: rgb(0,212,255);
}
Label {
    height: 1;
    color: rgb(0,212,255);
}
Checkbox {
    height: 1;
    color: rgb(224,224,224);
}
Switch {
    height: 1;
    color: rgb(224,224,224);
}
RadioSet {
    height: 3;
    color: rgb(224,224,224);
}
"#;

// ---- Tab pane widgets ----

/// Controls tab: form widgets composed as children.
struct ControlsPane;

impl Widget for ControlsPane {
    fn widget_type_name(&self) -> &'static str {
        "ControlsPane"
    }

    fn compose(&self) -> Vec<Box<dyn Widget>> {
        vec![
            Box::new(Label::new("Form Controls")),
            Box::new(Input::new("Type something...")),
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

    fn render(&self, _ctx: &AppContext, _area: Rect, _buf: &mut Buffer) {
        // Container only — children render via compose tree
    }
}

/// Data tab: table, progress bar, sparkline.
struct DataPane;

impl Widget for DataPane {
    fn widget_type_name(&self) -> &'static str {
        "DataPane"
    }

    fn compose(&self) -> Vec<Box<dyn Widget>> {
        let mut table = DataTable::new(vec![
            ColumnDef::new("Widget").with_width(20),
            ColumnDef::new("Status").with_width(12),
            ColumnDef::new("Version").with_width(10),
        ]);
        table.add_row(vec!["Label".into(), "Stable".into(), "v1.0".into()]);
        table.add_row(vec!["Button".into(), "Stable".into(), "v1.0".into()]);
        table.add_row(vec!["Input".into(), "Stable".into(), "v1.0".into()]);
        table.add_row(vec!["Checkbox".into(), "Stable".into(), "v1.0".into()]);
        table.add_row(vec!["DataTable".into(), "Stable".into(), "v1.0".into()]);

        vec![
            Box::new(Label::new("Data Widgets")),
            Box::new(table),
            Box::new(Label::new("Build progress: 65%")),
            Box::new(ProgressBar::new(0.65)),
            Box::new(Label::new("CPU activity:")),
            Box::new(Sparkline::new(vec![
                2.0, 4.0, 3.0, 7.0, 5.0, 6.0, 8.0, 4.0, 3.0, 5.0,
                7.0, 6.0, 4.0, 2.0, 5.0, 8.0, 7.0, 3.0, 6.0, 4.0,
            ])),
        ]
    }

    fn render(&self, _ctx: &AppContext, _area: Rect, _buf: &mut Buffer) {
        // Container only
    }
}

/// Lists tab: list view and log side by side.
struct ListsPane;

impl Widget for ListsPane {
    fn widget_type_name(&self) -> &'static str {
        "ListsPane"
    }

    fn compose(&self) -> Vec<Box<dyn Widget>> {
        let log = Log::new();
        log.push_line("[INFO]  server started on port 8080".to_string());
        log.push_line("[DEBUG] loading configuration file".to_string());
        log.push_line("[INFO]  database connection established".to_string());
        log.push_line("[WARN]  cache miss rate above threshold".to_string());
        log.push_line("[INFO]  request GET /api/widgets 200 OK".to_string());
        log.push_line("[DEBUG] widget tree recomposed, 12 nodes".to_string());
        log.push_line("[INFO]  request POST /api/submit 201 Created".to_string());
        log.push_line("[INFO]  CSS stylesheet reloaded".to_string());
        log.push_line("[WARN]  memory usage at 72%".to_string());
        log.push_line("[INFO]  reactive effect batched 3 updates".to_string());

        vec![
            Box::new(ListView::new(vec![
                "apple".to_string(),
                "banana".to_string(),
                "cherry".to_string(),
                "date".to_string(),
                "elderberry".to_string(),
                "fig".to_string(),
                "grape".to_string(),
                "honeydew".to_string(),
            ])),
            Box::new(log),
        ]
    }

    fn render(&self, _ctx: &AppContext, _area: Rect, _buf: &mut Buffer) {
        // Container only
    }
}

// ---- Top-level screen ----

struct DemoScreen;

impl Widget for DemoScreen {
    fn widget_type_name(&self) -> &'static str {
        "DemoScreen"
    }

    fn compose(&self) -> Vec<Box<dyn Widget>> {
        let tabbed = TabbedContent::new(
            vec![
                "Controls".to_string(),
                "Data".to_string(),
                "Lists".to_string(),
            ],
            vec![
                Box::new(ControlsPane),
                Box::new(DataPane),
                Box::new(ListsPane),
            ],
        );

        vec![
            Box::new(Header::new("textual-rs Widget Showcase").with_subtitle("Tab to navigate | q to quit")),
            Box::new(tabbed),
            Box::new(Footer),
        ]
    }

    fn render(&self, _ctx: &AppContext, _area: Rect, _buf: &mut Buffer) {
        // Container only — children do the rendering.
    }
}

fn main() -> anyhow::Result<()> {
    let mut app = App::new(|| Box::new(DemoScreen)).with_css(DEMO_CSS);
    app.run()
}
