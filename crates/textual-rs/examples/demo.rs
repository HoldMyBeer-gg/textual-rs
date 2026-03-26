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
    dock: top;
    height: 1;
    background: rgb(18,18,26);
    color: rgb(0,255,163);
}
Footer {
    dock: bottom;
    height: 1;
    background: rgb(18,18,26);
    color: rgb(224,224,224);
}
TabbedContent {
    flex-grow: 1;
}
Button {
    border: heavy;
    min-width: 16;
    height: 3;
}
Input {
    border: rounded;
    height: 3;
}
DataTable {
    border: rounded;
    min-height: 8;
}
ListView {
    border: rounded;
    min-height: 6;
    flex-grow: 1;
}
Log {
    border: rounded;
    min-height: 6;
    flex-grow: 1;
}
ProgressBar {
    height: 1;
}
Sparkline {
    height: 1;
}
"#;

// ---- Tab pane widgets ----

/// Controls tab: form widgets.
struct ControlsPane;

impl Widget for ControlsPane {
    fn widget_type_name(&self) -> &'static str {
        "ControlsPane"
    }

    fn render(&self, ctx: &AppContext, area: Rect, buf: &mut Buffer) {
        if area.height == 0 || area.width == 0 {
            return;
        }

        let mut y = area.y;

        // Section label
        let label = Label::new("-- Form Controls --");
        if y < area.y + area.height {
            label.render(ctx, Rect { x: area.x, y, width: area.width, height: 1 }, buf);
            y += 1;
        }

        // Input field
        if y + 2 < area.y + area.height {
            let input = Input::new("Type something...");
            input.render(ctx, Rect { x: area.x, y, width: area.width.min(40), height: 3 }, buf);
            y += 3;
        }

        // Checkbox
        if y < area.y + area.height {
            let cb = Checkbox::new("Enable notifications", true);
            cb.render(ctx, Rect { x: area.x, y, width: area.width, height: 1 }, buf);
            y += 1;
        }

        // Switch
        if y < area.y + area.height {
            let sw = Switch::new(false);
            sw.render(ctx, Rect { x: area.x, y, width: 20, height: 1 }, buf);
            y += 2;
        }

        // RadioSet
        if y + 4 < area.y + area.height {
            let radio = RadioSet::new(vec![
                "Option A".to_string(),
                "Option B".to_string(),
                "Option C".to_string(),
            ]);
            radio.render(ctx, Rect { x: area.x, y, width: area.width, height: 3 }, buf);
            y += 4;
        }

        // Buttons
        if y + 2 < area.y + area.height {
            let btn_primary = Button::new("Submit").with_variant(ButtonVariant::Primary);
            btn_primary.render(ctx, Rect { x: area.x, y, width: 18, height: 3 }, buf);
            if area.x + 20 + 18 < area.x + area.width {
                let btn_default = Button::new("Cancel");
                btn_default.render(ctx, Rect { x: area.x + 20, y, width: 18, height: 3 }, buf);
            }
        }
    }
}

/// Data tab: table, progress bar, sparkline.
struct DataPane;

impl Widget for DataPane {
    fn widget_type_name(&self) -> &'static str {
        "DataPane"
    }

    fn render(&self, ctx: &AppContext, area: Rect, buf: &mut Buffer) {
        if area.height == 0 || area.width == 0 {
            return;
        }

        let mut y = area.y;

        // Section label
        let label = Label::new("-- Data Display --");
        if y < area.y + area.height {
            label.render(ctx, Rect { x: area.x, y, width: area.width, height: 1 }, buf);
            y += 1;
        }

        // DataTable
        let table_height = 8u16.min(area.height.saturating_sub(y.saturating_sub(area.y) + 4));
        if table_height >= 3 {
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
            table.render(ctx, Rect { x: area.x, y, width: area.width.min(50), height: table_height }, buf);
            y += table_height + 1;
        }

        // Progress bar label + bar
        if y < area.y + area.height {
            let lbl = Label::new("Build progress: 65%");
            lbl.render(ctx, Rect { x: area.x, y, width: area.width, height: 1 }, buf);
            y += 1;
        }
        if y < area.y + area.height {
            let progress = ProgressBar::new(0.65);
            progress.render(ctx, Rect { x: area.x, y, width: area.width.min(50), height: 1 }, buf);
            y += 2;
        }

        // Sparkline label + sparkline
        if y < area.y + area.height {
            let lbl = Label::new("CPU activity:");
            lbl.render(ctx, Rect { x: area.x, y, width: area.width, height: 1 }, buf);
            y += 1;
        }
        if y < area.y + area.height {
            let sparkline = Sparkline::new(vec![
                2.0, 4.0, 3.0, 7.0, 5.0, 6.0, 8.0, 4.0, 3.0, 5.0,
                7.0, 6.0, 4.0, 2.0, 5.0, 8.0, 7.0, 3.0, 6.0, 4.0,
            ]);
            sparkline.render(ctx, Rect { x: area.x, y, width: area.width.min(50), height: 1 }, buf);
        }
    }
}

/// Lists tab: list view and log side by side.
struct ListsPane;

impl Widget for ListsPane {
    fn widget_type_name(&self) -> &'static str {
        "ListsPane"
    }

    fn render(&self, ctx: &AppContext, area: Rect, buf: &mut Buffer) {
        if area.height == 0 || area.width == 0 {
            return;
        }

        let half_width = area.width / 2;

        // ListView on left
        let list = ListView::new(vec![
            "apple".to_string(),
            "banana".to_string(),
            "cherry".to_string(),
            "date".to_string(),
            "elderberry".to_string(),
            "fig".to_string(),
            "grape".to_string(),
            "honeydew".to_string(),
        ]);
        list.render(ctx, Rect { x: area.x, y: area.y, width: half_width.saturating_sub(1), height: area.height }, buf);

        // Log on right
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
        log.render(ctx, Rect { x: area.x + half_width, y: area.y, width: area.width - half_width, height: area.height }, buf);
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
            Box::new(Footer),
            Box::new(tabbed),
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
