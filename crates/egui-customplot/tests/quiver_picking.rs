mod support;

use egui::{Color32, Context, Event, IdSalt};
use egui_customplot::{
    Axis, AxisScale, ColorScale, GridData, Heatmap, Line, LinePattern, Plot, PlotBounds, PlotData,
    PlotLayer, PlotState, Quiver, QuiverArrow, QuiverData, VectorAnchor, YAxis,
};
use support::run;

#[test]
fn origin_and_true_endpoint_pick_original_vector_record_not_shaft_wing_or_clip_intersection() {
    let ctx = Context::default();
    let records = vec![
        QuiverArrow::new([f64::NAN; 2], [1.0; 2]),
        QuiverArrow::new([0.5; 2], [0.0; 2]),
        QuiverArrow::new([0.2, 0.3], [0.6, 0.0]),
        QuiverArrow::new([-1.0, 0.7], [3.0, 0.0]),
    ];
    let data = QuiverData::new(records);
    let mut state = PlotState::default();
    state.set_bounds(PlotBounds::new([0.0; 2], [1.0; 2]).unwrap());
    let layer = Quiver::new(&data)
        .name("field")
        .id_salt("q")
        .head_size(20.0, 16.0)
        .into();
    let first = run(&ctx, vec![], false, |ui| {
        Plot::new("p").show_layers(ui, &mut state, &[layer])
    });
    for (point, anchor) in [
        ([0.2, 0.3], VectorAnchor::Origin),
        ([0.8, 0.3], VectorAnchor::Endpoint),
    ] {
        let pos = first.transform.position_from_point(point);
        let out = run(&ctx, vec![Event::PointerMoved(pos)], true, |ui| {
            Plot::new("p")
                .pick_radius(0.5)
                .show_layers(ui, &mut state, &[layer])
        });
        let vector = out.hovered_vector.unwrap();
        assert_eq!(vector.arrow, data.arrows()[2]);
        assert_eq!(vector.arrow_index, 2);
        assert_eq!(vector.anchor, anchor);
        assert_eq!(vector.layer_id, IdSalt::new("q"));
        assert_eq!(out.hovered_point.unwrap().point, point);
        let text = out.hover_text.unwrap();
        assert!(text.contains("field") && text.contains("vector=(") && text.contains("end=("));
        assert_eq!(out.diagnostics.visited_pick_quiver_arrows, 4);
    }
    let end = first.transform.position_from_point([0.8, 0.3]);
    let wing = end + egui::vec2(-20.0, 8.0);
    for pos in [[0.5, 0.3], [0.0, 0.7], [1.0, 0.7], [0.5; 2]]
        .map(|p| first.transform.position_from_point(p))
        .into_iter()
        .chain([wing])
    {
        let out = run(&ctx, vec![Event::PointerMoved(pos)], false, |ui| {
            Plot::new("p")
                .pick_radius(0.5)
                .show_layers(ui, &mut state, &[layer])
        });
        assert!(out.hovered_vector.is_none() && out.hovered_point.is_none());
    }
}

#[test]
fn visibility_and_numeric_rejections_match_rendering_even_when_heads_are_disabled() {
    let ctx = Context::default();
    let data = QuiverData::new(vec![QuiverArrow::new([0.2; 2], [0.6; 2])]);
    let base = Quiver::new(&data);
    for (quiver, visible) in [
        (base, true),
        (base.show_heads(false), true),
        (base.head_size(f32::NAN, 8.0), true),
        (base.width(0.0), false),
        (base.width(-1.0), false),
        (base.width(f32::NAN), false),
        (base.color(Color32::TRANSPARENT), false),
        (
            base.pattern(LinePattern::Dashed {
                length: 0.0,
                gap: 2.0,
            }),
            false,
        ),
    ] {
        let mut state = PlotState::default();
        let first = run(&ctx, vec![], false, |ui| {
            Plot::new("p").show_layers(ui, &mut state, &[quiver.into()])
        });
        let pos = first.transform.position_from_point([0.2; 2]);
        let out = run(&ctx, vec![Event::PointerMoved(pos)], false, |ui| {
            Plot::new("p").show_layers(ui, &mut state, &[quiver.into()])
        });
        assert_eq!(out.hovered_vector.is_some(), visible);
        assert_eq!(out.diagnostics.output_quiver_shafts > 0, visible);
    }
    let extreme = QuiverData::new(vec![QuiverArrow::new([0.5; 2], [1e100, 0.0])]);
    let mut state = PlotState::default();
    state.set_bounds(PlotBounds::new([0.0; 2], [1.0; 2]).unwrap());
    let first = run(&ctx, vec![], false, |ui| {
        Plot::new("p").show_layers(ui, &mut state, &[Quiver::new(&extreme).into()])
    });
    let pos = first.transform.position_from_point([0.5; 2]);
    let out = run(&ctx, vec![Event::PointerMoved(pos)], false, |ui| {
        Plot::new("p").show_layers(ui, &mut state, &[Quiver::new(&extreme).into()])
    });
    assert_eq!(out.diagnostics.numeric_rejections, 1);
    assert!(out.hovered_vector.is_none());
}

#[test]
fn right_log_reversed_raw_anchors_keep_joint_domain_and_existing_formatter_contract() {
    let ctx = Context::default();
    let left = PlotData::new(vec![[1.0, -1.0], [10.0, 1.0]]);
    let data = QuiverData::new(vec![
        QuiverArrow::new([2.0, 10.0], [4.0, 90.0]),
        QuiverArrow::new([2.0, 10.0], [4.0, -20.0]),
    ]);
    let mut state = PlotState::default();
    let layers = [
        Line::new(&left).into(),
        Quiver::new(&data).y_axis(YAxis::Right).into(),
    ];
    let plot = || {
        Plot::new("p")
            .x_axis(Axis::new().scale(AxisScale::Log10).reversed(true))
            .right_y_axis(Axis::new().scale(AxisScale::Log10).reversed(true))
            .hover_formatter(|info| {
                let p = info.hovered_point.unwrap();
                format!(
                    "raw={} axis={:?} point={:?}",
                    p.point_index, p.y_axis, p.point
                )
            })
    };
    let first = run(&ctx, vec![], false, |ui| {
        plot().show_layers(ui, &mut state, &layers)
    });
    assert!(first.transform.bounds().max()[1] < 2.0);
    assert_eq!(first.diagnostics.domain_invalid_quiver_arrows, 1);
    for point in [[2.0, 10.0], [6.0, 100.0]] {
        let pos = first.right_transform.unwrap().position_from_point(point);
        let out = run(&ctx, vec![Event::PointerMoved(pos)], true, |ui| {
            plot().show_layers(ui, &mut state, &layers)
        });
        let hit = out.hovered_vector.unwrap();
        assert_eq!(hit.y_axis, YAxis::Right);
        assert_eq!(hit.arrow_index, 0);
        assert_eq!(
            out.hover_text.unwrap(),
            format!("raw=0 axis=Right point={point:?}")
        );
    }
}

#[test]
fn nearest_distance_and_first_submission_ties_respect_filled_layer_occlusion() {
    let ctx = Context::default();
    let data = QuiverData::new(vec![QuiverArrow::new([0.2; 2], [0.6; 2])]);
    let points = PlotData::new(vec![[0.2; 2], [0.8; 2]]);
    let grid = GridData::new(vec![0.0, 1.0], vec![0.0, 1.0], vec![0.5]).unwrap();
    let q: PlotLayer<'_> = Quiver::new(&data).id_salt("q").into();
    let line = Line::new(&points).id_salt("line").into();
    let cell = Heatmap::new(&grid, ColorScale::new([0.0, 1.0]).unwrap()).into();
    for (layers, expected) in [
        (vec![q, line], Some("q")),
        (vec![line, q], Some("line")),
        (vec![q, cell], None),
        (vec![cell, q], Some("q")),
    ] {
        let mut state = PlotState::default();
        let first = run(&ctx, vec![], false, |ui| {
            Plot::new("p").show_layers(ui, &mut state, &layers)
        });
        let pos = first.transform.position_from_point([0.2; 2]);
        let out = run(&ctx, vec![Event::PointerMoved(pos)], false, |ui| {
            Plot::new("p").show_layers(ui, &mut state, &layers)
        });
        assert_eq!(
            out.hovered_point.as_ref().map(|p| p.series_id),
            expected.map(IdSalt::new)
        );
        assert_eq!(out.hovered_cell.is_some(), expected.is_none());
    }
}
