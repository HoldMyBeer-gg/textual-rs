//! Radio button and radio set widgets for single-selection from a group.
use crossterm::event::{KeyCode, KeyModifiers};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use reactive_graph::prelude::*;
use reactive_graph::signal::ArcRwSignal;
use std::cell::Cell;

use super::context::AppContext;
use super::{EventPropagation, Widget, WidgetId};
use crate::event::keybinding::KeyBinding;
use crate::reactive::Reactive;

/// Messages emitted by RadioButton and RadioSet.
pub mod messages {
    use crate::event::message::Message;

    /// Emitted by a RadioButton when it is selected.
    /// Includes the source widget ID so RadioSet can identify which button fired.
    pub struct RadioButtonChanged {
        /// Whether the button is now checked (always `true` for radio buttons).
        pub checked: bool,
        /// The widget ID of the RadioButton that was selected.
        pub source_id: super::WidgetId,
    }
    impl Message for RadioButtonChanged {}

    /// Emitted by a RadioSet when the selection changes.
    pub struct RadioSetChanged {
        /// Zero-based index of the newly selected button.
        pub index: usize,
        /// Label of the newly selected button.
        pub value: String,
    }
    impl Message for RadioSetChanged {}
}

/// A single radio button that can be checked. Part of a RadioSet for mutual exclusion.
///
/// Renders as `(●) label` when checked and `( ) label` when unchecked.
pub struct RadioButton {
    /// Whether this radio button is currently selected.
    pub checked: Reactive<bool>,
    /// Label text rendered after the indicator.
    pub label: String,
    /// Shared ArcRwSignal used by RadioSet to uncheck this button without downcasting.
    pub(crate) signal: ArcRwSignal<bool>,
    own_id: Cell<Option<WidgetId>>,
}

impl RadioButton {
    /// Create a new RadioButton with a label and initial checked state.
    pub fn new(label: impl Into<String>, checked: bool) -> Self {
        let signal = ArcRwSignal::new(checked);
        Self {
            checked: Reactive::new(checked),
            label: label.into(),
            signal,
            own_id: Cell::new(None),
        }
    }

    /// Create a RadioButton sharing an existing ArcRwSignal for state tracking.
    /// Used by RadioSet so it can uncheck buttons without downcasting.
    pub(crate) fn with_signal(
        label: impl Into<String>,
        checked: bool,
        signal: ArcRwSignal<bool>,
    ) -> Self {
        // Sync signal to initial value
        signal.set(checked);
        Self {
            checked: Reactive::new(checked),
            label: label.into(),
            signal,
            own_id: Cell::new(None),
        }
    }
}

static RADIO_BUTTON_BINDINGS: &[KeyBinding] = &[
    KeyBinding {
        key: KeyCode::Char(' '),
        modifiers: KeyModifiers::NONE,
        action: "select",
        description: "Select",
        show: true,
    },
    KeyBinding {
        key: KeyCode::Enter,
        modifiers: KeyModifiers::NONE,
        action: "select",
        description: "Select",
        show: false,
    },
];

impl Widget for RadioButton {
    fn widget_type_name(&self) -> &'static str {
        "RadioButton"
    }

    fn can_focus(&self) -> bool {
        true
    }

    fn default_css() -> &'static str
    where
        Self: Sized,
    {
        "RadioButton { height: 1; }"
    }

    fn on_mount(&self, id: WidgetId) {
        self.own_id.set(Some(id));
    }

    fn on_unmount(&self, _id: WidgetId) {
        self.own_id.set(None);
    }

    fn key_bindings(&self) -> &[KeyBinding] {
        RADIO_BUTTON_BINDINGS
    }

    fn click_action(&self) -> Option<&str> {
        Some("select")
    }

    fn on_action(&self, action: &str, ctx: &AppContext) {
        if action == "select" {
            // Radio buttons only turn ON, never toggle off
            self.checked.set(true);
            self.signal.set(true);
            if let Some(id) = self.own_id.get() {
                ctx.post_message(
                    id,
                    messages::RadioButtonChanged {
                        checked: true,
                        source_id: id,
                    },
                );
            }
        }
    }

    fn render(&self, ctx: &AppContext, area: Rect, buf: &mut Buffer) {
        use ratatui::style::Color;

        if area.height == 0 || area.width == 0 {
            return;
        }

        let base_style = self
            .own_id
            .get()
            .map(|id| ctx.text_style(id))
            .unwrap_or_default();

        // Read from self.signal (shared with RadioSet) so mutual exclusion is reflected.
        let checked = self.signal.get_untracked();
        // Color-differentiated: green filled dot when selected, dim empty when not
        let (indicator, ind_style) = if checked {
            ("◉", base_style.fg(Color::Rgb(0, 255, 163)))
        } else {
            ("○", base_style.fg(Color::Rgb(100, 100, 110)))
        };
        buf.set_string(area.x, area.y, indicator, ind_style);

        // Label after indicator
        if area.width > 2 {
            let label_text: String = self.label.chars().take((area.width - 2) as usize).collect();
            buf.set_string(area.x + 2, area.y, &label_text, base_style);
        }
    }
}

/// A group of RadioButtons that enforces mutual exclusion.
///
/// When one RadioButton is selected, all others are automatically deselected.
/// Emits `messages::RadioSetChanged` with the index and label of the newly selected button.
pub struct RadioSet {
    /// The signal backing for each button — shared with buttons in compose() for mutual exclusion.
    signals: Vec<ArcRwSignal<bool>>,
    /// Labels corresponding to each button.
    labels: Vec<String>,
    /// Index of the currently selected button.
    pub selected: Reactive<usize>,
    own_id: Cell<Option<WidgetId>>,
}

impl RadioSet {
    /// Create a new RadioSet from a list of labels.
    /// The first button is selected by default.
    pub fn new(labels: Vec<String>) -> Self {
        let n = labels.len();
        let signals: Vec<ArcRwSignal<bool>> = (0..n).map(|i| ArcRwSignal::new(i == 0)).collect();
        Self {
            signals,
            labels,
            selected: Reactive::new(0),
            own_id: Cell::new(None),
        }
    }
}

impl Widget for RadioSet {
    fn widget_type_name(&self) -> &'static str {
        "RadioSet"
    }

    fn can_focus(&self) -> bool {
        // RadioSet itself is not focusable — its RadioButton children are
        false
    }

    fn default_css() -> &'static str
    where
        Self: Sized,
    {
        "RadioSet { layout: vertical; }"
    }

    fn on_mount(&self, id: WidgetId) {
        self.own_id.set(Some(id));
    }

    fn on_unmount(&self, _id: WidgetId) {
        self.own_id.set(None);
    }

    fn compose(&self) -> Vec<Box<dyn Widget>> {
        // Create RadioButtons sharing our signals so we can uncheck them later
        self.labels
            .iter()
            .enumerate()
            .map(|(i, label)| {
                let signal = self.signals[i].clone();
                let checked = i == 0;
                Box::new(RadioButton::with_signal(label.clone(), checked, signal))
                    as Box<dyn Widget>
            })
            .collect()
    }

    fn on_event(&self, event: &dyn std::any::Any, ctx: &AppContext) -> EventPropagation {
        if let Some(changed) = event.downcast_ref::<messages::RadioButtonChanged>() {
            if !changed.checked {
                // Only handle selections (checked = true)
                return EventPropagation::Continue;
            }

            // Find which child index this source corresponds to
            let own_id = match self.own_id.get() {
                Some(id) => id,
                None => return EventPropagation::Continue,
            };

            let children = match ctx.children.get(own_id) {
                Some(c) => c,
                None => return EventPropagation::Continue,
            };

            // Find the selected index by matching source_id to children
            let selected_idx = children
                .iter()
                .position(|&child_id| child_id == changed.source_id);

            if let Some(idx) = selected_idx {
                // Uncheck all OTHER buttons via their shared signals.
                // RadioButton.render() reads from self.signal (the shared ArcRwSignal),
                // so setting it here will be reflected on next render.
                for (i, signal) in self.signals.iter().enumerate() {
                    if i != idx {
                        signal.set(false);
                    }
                }

                // Update selected index
                self.selected.set(idx);

                // Post RadioSetChanged
                if let Some(own_id) = self.own_id.get() {
                    let label = self.labels[idx].clone();
                    ctx.post_message(
                        own_id,
                        messages::RadioSetChanged {
                            index: idx,
                            value: label,
                        },
                    );
                }

                return EventPropagation::Stop;
            }
        }
        EventPropagation::Continue
    }

    fn render(&self, _ctx: &AppContext, area: Rect, buf: &mut Buffer) {
        if area.height == 0 || area.width == 0 {
            return;
        }
        // RadioSet itself renders nothing — its RadioButton children render themselves
        // The layout engine positions them via compose()/vertical layout
        let _ = (area, buf);
    }
}
