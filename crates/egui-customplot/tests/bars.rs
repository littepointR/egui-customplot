mod support;

use egui::{Color32, Context, Event, IdSalt, Shape, Stroke};
use egui_customplot::{
    Axis, AxisScale, Bar, BarData, BarOrientation, Bars, ColorScale, GridData, Heatmap, Line, Plot,
    PlotBounds, PlotData, PlotLayer, PlotState, YAxis,
};
use support::{capture, run};

#[test]
fn signed_rectangles_keep_data_width_baselines_direction_and_original_order() {
    let ctx = Context::default();
    let data = BarData::new(vec![
        Bar::new(2.0, 5.0).baseline(1.0).width(2.0),
        Bar::new(6.0, -3.0).baseline(1.0),
    ]);
    for orientation in [BarOrientation::Vertical, BarOrientation::Horizontal] {
        let mut state = PlotState::default();
        let (out, output) = capture(&ctx, vec![], true, |ui| {
            Plot::new("p").show_layers(
                ui,
                &mut state,
                &[Bars::new(&data)
                    .orientation(orientation)
                    .color(Color32::RED)
                    .into()],
            )
        });
        assert_eq!(out.diagnostics.output_bars, 2);
        assert_eq!(out.diagnostics.input_bars, 2);
        let rects = output
            .shapes
            .iter()
            .filter_map(|s| match &s.shape {
                Shape::Rect(r) if r.fill == Color32::RED => Some(r.rect),
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(rects.len(), 2);
        for (i, mut lo, mut hi) in [(0, [1.0, 1.0], [3.0, 5.0]), (1, [5.6, -3.0], [6.4, 1.0])] {
            if orientation == BarOrientation::Horizontal {
                lo.swap(0, 1);
                hi.swap(0, 1);
            }
            let expected = egui::Rect::from_two_pos(
                out.transform.position_from_point(lo),
                out.transform.position_from_point(hi),
            );
            assert_eq!(rects[i], expected);
        }
        output.drop_without_applying_deltas();
    }
}

#[test]
fn invalid_records_keep_raw_indices_and_log_domain_rejects_the_whole_bar() {
    let ctx = Context::default();
    let data = BarData::new(vec![
        Bar::new(f64::NAN, 2.0),
        Bar::new(2.0, 4.0),
        Bar::new(4.0, 10.0).baseline(1.0).width(-1.0),
        Bar::new(6.0, 20.0).baseline(1.0).width(2.0),
        Bar::new(1e300, 3.0).width(0.01),
    ]);
    let mut state = PlotState::default();
    let plot = || Plot::new("p").y_axis(Axis::new().scale(AxisScale::Log10).reversed(true));
    let out = run(&ctx, vec![], false, |ui| {
        plot().show_layers(ui, &mut state, &[Bars::new(&data).into()])
    });
    assert_eq!(
        (
            out.diagnostics.invalid_bars,
            out.diagnostics.domain_invalid_bars
        ),
        (3, 1)
    );
    assert_eq!(out.diagnostics.output_bars, 1);
    let pos = out.transform.position_from_point([6.0, 10.0]);
    let out = run(&ctx, vec![Event::PointerMoved(pos)], true, |ui| {
        plot().show_layers(ui, &mut state, &[Bars::new(&data).into()])
    });
    assert_eq!(out.hovered_bar.as_ref().unwrap().bar_index, 3);
    assert_eq!(out.hovered_bar.unwrap().bar, data.bars()[3]);
    assert!(out.hover_text.unwrap().contains("baseline="));
}

#[test]
fn fills_and_original_borders_are_clipped_without_fabricating_view_edges() {
    let ctx = Context::default();
    let data = BarData::new(vec![
        Bar::new(0.0, 1e300).baseline(-1e300).width(2e300),
        Bar::new(0.0, 0.0),
    ]);
    let mut state = PlotState::default();
    state.set_bounds(PlotBounds::new([-1.0; 2], [1.0; 2]).unwrap());
    let (out, output) = capture(&ctx, vec![], true, |ui| {
        Plot::new("p").show_layers(
            ui,
            &mut state,
            &[Bars::new(&data)
                .color(Color32::RED)
                .border(Stroke::new(3.0, Color32::GREEN))
                .into()],
        )
    });
    assert_eq!(
        out.diagnostics.output_bars, 1,
        "zero-area bars emit no body or outline"
    );
    assert!(output.shapes.iter().any(|s| matches!(&s.shape, Shape::Rect(r) if r.fill == Color32::RED && r.rect == out.transform.rect())));
    assert!(!output.shapes.iter().any(
        |s| matches!(&s.shape, Shape::LineSegment { stroke, .. } if stroke.color == Color32::GREEN)
    ));
    output.drop_without_applying_deltas();
}

#[test]
fn filled_layer_arbitration_keeps_topmost_bars_cells_and_later_point_ties() {
    let ctx = Context::default();
    let bars = BarData::new(vec![Bar::new(1.0, 2.0).width(1.5); 2]);
    let grid = GridData::new(vec![0.0, 2.0], vec![0.0, 2.0], vec![1.0]).unwrap();
    let points = PlotData::new(vec![[1.0; 2]]);
    let bar: PlotLayer<'_> = Bars::new(&bars).id_salt("bars").into();
    let cell = Heatmap::new(&grid, ColorScale::new([0.0, 2.0]).unwrap()).into();
    let point = Line::new(&points).into();
    for (layers, winner) in [
        (vec![cell, point, bar], 0),
        (vec![bar, point, cell], 1),
        (vec![cell, bar, point], 2),
        (vec![bar, point, point], 2),
    ] {
        let mut state = PlotState::default();
        let first = run(&ctx, vec![], false, |ui| {
            Plot::new("p").show_layers(ui, &mut state, &layers)
        });
        let pos = first.transform.position_from_point([1.0; 2]);
        let out = run(&ctx, vec![Event::PointerMoved(pos)], true, |ui| {
            Plot::new("p").show_layers(ui, &mut state, &layers)
        });
        assert_eq!(out.hovered_bar.is_some(), winner == 0);
        assert_eq!(out.hovered_cell.is_some(), winner == 1);
        assert_eq!(out.hovered_point.is_some(), winner == 2);
        if let Some(bar) = out.hovered_bar {
            assert_eq!(bar.bar_index, 1);
        }
        if let Some(point) = out.hovered_point {
            assert_eq!(
                point.series_index,
                if layers.len() == 3 && matches!(layers[1], PlotLayer::Series(_)) {
                    1
                } else {
                    2
                }
            );
        }
    }
}

#[test]
fn transparent_outline_hidden_and_invalid_styles_match_picking() {
    let ctx = Context::default();
    let data = BarData::new(vec![Bar::new(1.0, 2.0)]);
    for (color, border, should_pick) in [
        (Color32::TRANSPARENT, Stroke::NONE, false),
        (Color32::TRANSPARENT, Stroke::new(2.0, Color32::RED), false),
        (Color32::BLUE, Stroke::new(f32::NAN, Color32::RED), true),
    ] {
        let mut state = PlotState::default();
        let bars = Bars::new(&data).color(color).border(border).id_salt("b");
        let out = run(&ctx, vec![], false, |ui| {
            Plot::new("p").show_layers(ui, &mut state, &[bars.into()])
        });
        let pos = out.transform.position_from_point([1.0; 2]);
        let out = run(&ctx, vec![Event::PointerMoved(pos)], true, |ui| {
            Plot::new("p").show_layers(ui, &mut state, &[bars.into()])
        });
        assert_eq!(out.hovered_bar.is_some(), should_pick);
        if border.width.is_nan() {
            assert_eq!(out.diagnostics.numeric_rejections, 1);
        }
        state.set_series_visible(IdSalt::new("b"), false);
        let out = run(&ctx, vec![], false, |ui| {
            Plot::new("p").show_layers(ui, &mut state, &[bars.into()])
        });
        assert_eq!(out.diagnostics.visited_bars, 0);
        assert!(out.hovered_bar.is_none());
    }
}

#[test]
fn horizontal_bars_use_right_log_axis_for_bounds_raw_hover_and_formatter() {
    let ctx = Context::default();
    let data = BarData::new(vec![Bar::new(100.0, 3.0).width(100.0).baseline(1.0)]);
    let mut state = PlotState::default();
    let bars = Bars::new(&data)
        .orientation(BarOrientation::Horizontal)
        .y_axis(YAxis::Right);
    let plot = || {
        Plot::new("p")
            .right_y_axis(Axis::new().scale(AxisScale::Log10).reversed(true))
            .hover_formatter(|info| format!("raw={:?}", info.hovered_point.unwrap().point))
    };
    let first = run(&ctx, vec![], false, |ui| {
        plot().show_layers(ui, &mut state, &[bars.into()])
    });
    let pos = first
        .right_transform
        .unwrap()
        .position_from_point([2.0, 100.0]);
    let out = run(&ctx, vec![Event::PointerMoved(pos)], true, |ui| {
        plot().show_layers(ui, &mut state, &[bars.into()])
    });
    assert_eq!(out.hovered_bar.unwrap().y_axis, YAxis::Right);
    assert_eq!(out.hover_text.as_deref(), Some("raw=[3.0, 100.0]"));
    assert!(out.hovered_point.is_none());
}
