// TDD RED phase: tests for layout module
// These tests define the expected behavior for TaffyBridge, style_map, and MouseHitMap.
// They will fail until the implementation is complete (GREEN phase).

#[cfg(test)]
mod style_map_tests {
    use crate::css::types::*;
    use crate::layout::style_map::taffy_style_from_computed;
    use taffy::prelude::*;

    fn default_style() -> ComputedStyle {
        ComputedStyle::default()
    }

    #[test]
    fn default_computed_style_maps_to_flex_column() {
        let s = default_style();
        let ts = taffy_style_from_computed(&s);
        assert_eq!(ts.display, Display::Flex);
        // FlexDirection::Column is the default for vertical
        assert_eq!(ts.flex_direction, FlexDirection::Column);
    }

    #[test]
    fn length_dimension_maps_correctly() {
        let mut s = default_style();
        s.width = TcssDimension::Length(20.0);
        let ts = taffy_style_from_computed(&s);
        assert_eq!(ts.size.width, Dimension::length(20.0));
    }

    #[test]
    fn percent_dimension_maps_correctly() {
        let mut s = default_style();
        s.width = TcssDimension::Percent(50.0);
        let ts = taffy_style_from_computed(&s);
        // Taffy uses 0.0..1.0 range for percent
        assert_eq!(ts.size.width, Dimension::percent(0.5));
    }

    #[test]
    fn dock_top_maps_to_absolute_position_with_correct_insets() {
        let mut s = default_style();
        s.dock = Some(DockEdge::Top);
        s.height = TcssDimension::Length(1.0);
        let ts = taffy_style_from_computed(&s);
        assert_eq!(ts.position, Position::Absolute);
        // top=0, left=0, right=0, bottom=auto
        assert_eq!(ts.inset.top, LengthPercentageAuto::length(0.0));
        assert_eq!(ts.inset.left, LengthPercentageAuto::length(0.0));
        assert_eq!(ts.inset.right, LengthPercentageAuto::length(0.0));
        assert!(ts.inset.bottom.is_auto());
    }
}

#[cfg(test)]
mod bridge_tests {
    use crate::css::types::*;
    use crate::layout::bridge::TaffyBridge;
    use crate::widget::context::AppContext;
    use crate::widget::tree::{clear_dirty_subtree, mount_widget};
    use crate::widget::{Widget, WidgetId};
    use ratatui::buffer::Buffer;
    use ratatui::layout::Rect;

    struct TestWidget;
    impl Widget for TestWidget {
        fn render(&self, _ctx: &AppContext, _area: Rect, _buf: &mut Buffer) {}
        fn widget_type_name(&self) -> &'static str {
            "TestWidget"
        }
    }

    fn make_widget() -> Box<dyn Widget> {
        Box::new(TestWidget)
    }

    /// Helper: mount a widget, set its computed style, return id
    fn mount_with_style(
        ctx: &mut AppContext,
        parent: Option<WidgetId>,
        style: ComputedStyle,
    ) -> WidgetId {
        let id = mount_widget(make_widget(), parent, ctx);
        ctx.computed_styles[id] = style;
        id
    }

    #[test]
    fn taffy_bridge_new_creates_empty_bridge() {
        let bridge = TaffyBridge::new();
        assert!(bridge.layout_cache().is_empty());
    }

    #[test]
    fn vertical_flex_two_children_fills_height() {
        let mut ctx = AppContext::new();
        // Screen: 80x24, flex column (default)
        let screen_style = ComputedStyle {
            display: TcssDisplay::Flex,
            layout_direction: LayoutDirection::Vertical,
            width: TcssDimension::Length(80.0),
            height: TcssDimension::Length(24.0),
            ..Default::default()
        };
        let screen = mount_with_style(&mut ctx, None, screen_style);

        // Two children: each gets flex_grow=1 so they split equally
        let child1_style = ComputedStyle {
            flex_grow: 1.0,
            ..Default::default()
        };
        let child2_style = ComputedStyle {
            flex_grow: 1.0,
            ..Default::default()
        };
        let child1 = mount_with_style(&mut ctx, Some(screen), child1_style);
        let child2 = mount_with_style(&mut ctx, Some(screen), child2_style);

        let mut bridge = TaffyBridge::new();
        bridge.sync_subtree(screen, &ctx);
        bridge.compute_layout(screen, 80, 24, &ctx);

        let r1 = bridge.rect_for(child1).expect("child1 should have rect");
        let r2 = bridge.rect_for(child2).expect("child2 should have rect");

        assert_eq!(
            r1,
            Rect {
                x: 0,
                y: 0,
                width: 80,
                height: 12
            }
        );
        assert_eq!(
            r2,
            Rect {
                x: 0,
                y: 12,
                width: 80,
                height: 12
            }
        );
    }

    #[test]
    fn horizontal_flex_fractional_children_correct_widths() {
        let mut ctx = AppContext::new();
        // Screen: 80x24, flex row (horizontal)
        let screen_style = ComputedStyle {
            display: TcssDisplay::Flex,
            layout_direction: LayoutDirection::Horizontal,
            width: TcssDimension::Length(80.0),
            height: TcssDimension::Length(24.0),
            ..Default::default()
        };
        let screen = mount_with_style(&mut ctx, None, screen_style);

        // 3 children with flex_grow 1, 2, 1 → widths 20, 40, 20
        let c1 = mount_with_style(
            &mut ctx,
            Some(screen),
            ComputedStyle {
                flex_grow: 1.0,
                ..Default::default()
            },
        );
        let c2 = mount_with_style(
            &mut ctx,
            Some(screen),
            ComputedStyle {
                flex_grow: 2.0,
                ..Default::default()
            },
        );
        let c3 = mount_with_style(
            &mut ctx,
            Some(screen),
            ComputedStyle {
                flex_grow: 1.0,
                ..Default::default()
            },
        );

        let mut bridge = TaffyBridge::new();
        bridge.sync_subtree(screen, &ctx);
        bridge.compute_layout(screen, 80, 24, &ctx);

        let r1 = bridge.rect_for(c1).expect("c1 rect");
        let r2 = bridge.rect_for(c2).expect("c2 rect");
        let r3 = bridge.rect_for(c3).expect("c3 rect");

        assert_eq!(r1.width, 20, "c1 width should be 20");
        assert_eq!(r2.width, 40, "c2 width should be 40");
        assert_eq!(r3.width, 20, "c3 width should be 20");
        // Verify they're side by side
        assert_eq!(r1.x, 0);
        assert_eq!(r2.x, 20);
        assert_eq!(r3.x, 60);
    }

    #[test]
    fn grid_layout_2x2_correct_rects() {
        let mut ctx = AppContext::new();
        // Screen: 80x24, grid 2 cols 2 rows
        let screen_style = ComputedStyle {
            display: TcssDisplay::Grid,
            width: TcssDimension::Length(80.0),
            height: TcssDimension::Length(24.0),
            grid_columns: Some(vec![
                TcssDimension::Fraction(1.0),
                TcssDimension::Fraction(1.0),
            ]),
            grid_rows: Some(vec![
                TcssDimension::Fraction(1.0),
                TcssDimension::Fraction(1.0),
            ]),
            ..Default::default()
        };
        let screen = mount_with_style(&mut ctx, None, screen_style);

        // 4 children in grid cells
        let c1 = mount_with_style(&mut ctx, Some(screen), ComputedStyle::default());
        let c2 = mount_with_style(&mut ctx, Some(screen), ComputedStyle::default());
        let c3 = mount_with_style(&mut ctx, Some(screen), ComputedStyle::default());
        let c4 = mount_with_style(&mut ctx, Some(screen), ComputedStyle::default());

        let mut bridge = TaffyBridge::new();
        bridge.sync_subtree(screen, &ctx);
        bridge.compute_layout(screen, 80, 24, &ctx);

        let r1 = bridge.rect_for(c1).expect("c1 rect");
        let r2 = bridge.rect_for(c2).expect("c2 rect");
        let r3 = bridge.rect_for(c3).expect("c3 rect");
        let r4 = bridge.rect_for(c4).expect("c4 rect");

        // 2 columns: each 40 wide. 2 rows: each 12 tall.
        assert_eq!(
            r1,
            Rect {
                x: 0,
                y: 0,
                width: 40,
                height: 12
            }
        );
        assert_eq!(
            r2,
            Rect {
                x: 40,
                y: 0,
                width: 40,
                height: 12
            }
        );
        assert_eq!(
            r3,
            Rect {
                x: 0,
                y: 12,
                width: 40,
                height: 12
            }
        );
        assert_eq!(
            r4,
            Rect {
                x: 40,
                y: 12,
                width: 40,
                height: 12
            }
        );
    }

    #[test]
    fn dock_top_height_1_pins_to_top() {
        let mut ctx = AppContext::new();
        // Screen: 80x24, flex column (default)
        let screen_style = ComputedStyle {
            display: TcssDisplay::Flex,
            layout_direction: LayoutDirection::Vertical,
            width: TcssDimension::Length(80.0),
            height: TcssDimension::Length(24.0),
            ..Default::default()
        };
        let screen = mount_with_style(&mut ctx, None, screen_style);

        // Header: dock top, height 1
        let header_style = ComputedStyle {
            dock: Some(DockEdge::Top),
            height: TcssDimension::Length(1.0),
            width: TcssDimension::Auto,
            ..Default::default()
        };
        let header = mount_with_style(&mut ctx, Some(screen), header_style);

        // Body: fills remaining space
        let body_style = ComputedStyle {
            flex_grow: 1.0,
            ..Default::default()
        };
        let _body = mount_with_style(&mut ctx, Some(screen), body_style);

        let mut bridge = TaffyBridge::new();
        bridge.sync_subtree(screen, &ctx);
        bridge.compute_layout(screen, 80, 24, &ctx);

        let header_rect = bridge.rect_for(header).expect("header rect");
        assert_eq!(
            header_rect,
            Rect {
                x: 0,
                y: 0,
                width: 80,
                height: 1
            }
        );
    }

    #[test]
    fn fixed_width_and_auto_in_horizontal_flex() {
        let mut ctx = AppContext::new();
        let screen_style = ComputedStyle {
            display: TcssDisplay::Flex,
            layout_direction: LayoutDirection::Horizontal,
            width: TcssDimension::Length(80.0),
            height: TcssDimension::Length(24.0),
            ..Default::default()
        };
        let screen = mount_with_style(&mut ctx, None, screen_style);

        // fixed: 20 wide
        let fixed = mount_with_style(
            &mut ctx,
            Some(screen),
            ComputedStyle {
                width: TcssDimension::Length(20.0),
                ..Default::default()
            },
        );
        // auto: fills remainder with flex_grow
        let auto_fill = mount_with_style(
            &mut ctx,
            Some(screen),
            ComputedStyle {
                flex_grow: 1.0,
                ..Default::default()
            },
        );

        let mut bridge = TaffyBridge::new();
        bridge.sync_subtree(screen, &ctx);
        bridge.compute_layout(screen, 80, 24, &ctx);

        let r_fixed = bridge.rect_for(fixed).expect("fixed rect");
        let r_auto = bridge.rect_for(auto_fill).expect("auto rect");

        assert_eq!(r_fixed.width, 20);
        assert_eq!(r_auto.width, 60); // 80 - 20 = 60
    }

    #[test]
    fn percent_width_50_in_80_cols() {
        let mut ctx = AppContext::new();
        let screen_style = ComputedStyle {
            display: TcssDisplay::Flex,
            layout_direction: LayoutDirection::Horizontal,
            width: TcssDimension::Length(80.0),
            height: TcssDimension::Length(24.0),
            ..Default::default()
        };
        let screen = mount_with_style(&mut ctx, None, screen_style);

        let child = mount_with_style(
            &mut ctx,
            Some(screen),
            ComputedStyle {
                width: TcssDimension::Percent(50.0),
                ..Default::default()
            },
        );

        let mut bridge = TaffyBridge::new();
        bridge.sync_subtree(screen, &ctx);
        bridge.compute_layout(screen, 80, 24, &ctx);

        let r = bridge.rect_for(child).expect("child rect");
        assert_eq!(r.width, 40);
    }

    #[test]
    fn dirty_flag_prevents_sync_of_clean_subtree() {
        let mut ctx = AppContext::new();
        let screen_style = ComputedStyle {
            display: TcssDisplay::Flex,
            layout_direction: LayoutDirection::Vertical,
            width: TcssDimension::Length(80.0),
            height: TcssDimension::Length(24.0),
            ..Default::default()
        };
        let screen = mount_with_style(&mut ctx, None, screen_style);

        let child = mount_with_style(
            &mut ctx,
            Some(screen),
            ComputedStyle {
                flex_grow: 1.0,
                ..Default::default()
            },
        );

        let mut bridge = TaffyBridge::new();

        // First sync: all dirty
        bridge.sync_subtree(screen, &ctx);
        bridge.compute_layout(screen, 80, 24, &ctx);
        assert!(bridge.rect_for(child).is_some());

        // Mark clean, then change child style (but dirty_sync should skip it)
        clear_dirty_subtree(screen, &mut ctx);
        assert_eq!(ctx.dirty[screen], false);
        assert_eq!(ctx.dirty[child], false);

        // Change the child's style but don't mark dirty
        ctx.computed_styles[child] = ComputedStyle {
            flex_grow: 99.0, // bogus value
            ..Default::default()
        };

        // sync_dirty_subtree should skip clean widgets
        bridge.sync_dirty_subtree(screen, &ctx);
        bridge.compute_layout(screen, 80, 24, &ctx);

        // Layout should NOT have changed (child was not re-synced)
        // The child's taffy node still has the old style
        let r = bridge.rect_for(child).expect("child rect");
        // It should still be 24 tall (fill entire 24 rows since flex_grow=1 in column)
        assert_eq!(r.height, 24);
    }
}

#[cfg(test)]
mod hit_map_tests {
    use crate::layout::hit_map::MouseHitMap;
    use crate::widget::context::AppContext;
    use crate::widget::tree::mount_widget;
    use crate::widget::{Widget, WidgetId};
    use ratatui::buffer::Buffer;
    use ratatui::layout::Rect;
    use std::collections::HashMap;

    struct TestWidget;
    impl Widget for TestWidget {
        fn render(&self, _ctx: &AppContext, _area: Rect, _buf: &mut Buffer) {}
        fn widget_type_name(&self) -> &'static str {
            "TestWidget"
        }
    }

    fn make_ctx_with_two_widgets() -> (AppContext, WidgetId, WidgetId) {
        let mut ctx = AppContext::new();
        let w1 = mount_widget(Box::new(TestWidget), None, &mut ctx);
        let w2 = mount_widget(Box::new(TestWidget), None, &mut ctx);
        (ctx, w1, w2)
    }

    #[test]
    fn hit_test_returns_correct_widget_for_cell_inside_rect() {
        let (_, w1, w2) = make_ctx_with_two_widgets();
        let mut cache: HashMap<WidgetId, Rect> = HashMap::new();
        cache.insert(
            w1,
            Rect {
                x: 0,
                y: 0,
                width: 40,
                height: 12,
            },
        );
        cache.insert(
            w2,
            Rect {
                x: 40,
                y: 0,
                width: 40,
                height: 12,
            },
        );

        let order = vec![w1, w2];
        let hit_map = MouseHitMap::build(&order, &cache);

        // w1 occupies cols 0..40, rows 0..12
        assert_eq!(hit_map.hit_test(0, 0), Some(w1));
        assert_eq!(hit_map.hit_test(39, 11), Some(w1));

        // w2 occupies cols 40..80, rows 0..12
        assert_eq!(hit_map.hit_test(40, 0), Some(w2));
        assert_eq!(hit_map.hit_test(79, 11), Some(w2));
    }

    #[test]
    fn hit_test_returns_none_outside_all_rects() {
        let (_, w1, _w2) = make_ctx_with_two_widgets();
        let mut cache: HashMap<WidgetId, Rect> = HashMap::new();
        cache.insert(
            w1,
            Rect {
                x: 0,
                y: 0,
                width: 40,
                height: 12,
            },
        );

        let order = vec![w1];
        let hit_map = MouseHitMap::build(&order, &cache);

        // Outside widget rect
        assert_eq!(hit_map.hit_test(40, 0), None);
        assert_eq!(hit_map.hit_test(0, 12), None);
        assert_eq!(hit_map.hit_test(100, 100), None);
    }

    #[test]
    fn overlapping_rects_later_dfs_widget_wins() {
        let (_, w1, w2) = make_ctx_with_two_widgets();
        let mut cache: HashMap<WidgetId, Rect> = HashMap::new();
        // Both occupy same area — w2 is later in DFS so it wins
        cache.insert(
            w1,
            Rect {
                x: 0,
                y: 0,
                width: 40,
                height: 12,
            },
        );
        cache.insert(
            w2,
            Rect {
                x: 0,
                y: 0,
                width: 40,
                height: 12,
            },
        );

        let order = vec![w1, w2]; // w2 is later
        let hit_map = MouseHitMap::build(&order, &cache);

        assert_eq!(hit_map.hit_test(0, 0), Some(w2));
        assert_eq!(hit_map.hit_test(20, 6), Some(w2));
    }
}
