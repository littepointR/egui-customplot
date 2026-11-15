mod support;

use std::sync::Arc;

use egui::{Align2, Color32, Context, Shape, Stroke};
use egui_customplot::{Annotation, Axis, AxisScale, Line, Plot, PlotBounds, PlotData, PlotState};
use support::{capture, run};

#[test]
fn annotations_never_expand_auto_bounds_and_release_shared_labels() {
    let ctx = Context::default();
    let data = PlotData::new(vec![[1.0; 2], [2.0; 2]]);
    let label: Arc<str> = "outside".into();
    let weak = Arc::downgrade(&label);
    let annotations: Arc<[Annotation]> = vec![
        Annotation::text([1e10; 2], label.clone()),
        Annotation::x_line(-1e10, "left"),
        Annotation::y_region([1e8, f64::INFINITY], Color32::RED),
    ]
    .into();
    let mut state = PlotState::default();
    let first = run(&ctx, vec![], false, |ui| {
        Plot::new("p").show(ui, &mut state, &[Line::new(&data)])
    });
    let out = run(&ctx, vec![], true, |ui| {
        Plot::new("p")
            .annotations(annotations.clone())
            .show(ui, &mut state, &[Line::new(&data)])
    });
    assert_eq!(first.transform.bounds(), out.transform.bounds());
    assert_eq!(out.diagnostics.output_annotations, 0);
    assert_eq!(out.diagnostics.annotation_rejections, 0);
    assert_eq!(
        Arc::strong_count(&annotations),
        1,
        "PlotState must not retain annotations"
    );
    drop(label);
    drop(annotations);
    assert!(weak.upgrade().is_none());
}

#[test]
fn reference_lines_and_infinite_regions_follow_log_and_reversed_axes() {
    let ctx = Context::default();
    let x_fill = Color32::from_rgba_unmultiplied(15, 65, 125, 70);
    let y_fill = Color32::from_rgba_unmultiplied(145, 25, 75, 60);
    for reversed in [[false; 2], [true; 2], [true, false]] {
        let mut state = PlotState::default();
        state.set_bounds(PlotBounds::new([1.0; 2], [100.0; 2]).unwrap());
        let annotations: Arc<[Annotation]> = vec![
            Annotation::x_region([f64::NEG_INFINITY, 10.0], x_fill),
            Annotation::y_region([10.0, f64::INFINITY], y_fill),
            Annotation::x_line(10.0, "x = 10"),
            Annotation::y_line(10.0, "y = 10"),
            Annotation::text([10.0; 2], "center"),
        ]
        .into();
        let (result, out) = capture(&ctx, vec![], false, |ui| {
            Plot::new("p")
                .x_axis(Axis::new().scale(AxisScale::Log10).reversed(reversed[0]))
                .y_axis(Axis::new().scale(AxisScale::Log10).reversed(reversed[1]))
                .annotations(annotations.clone())
                .show(ui, &mut state, &[])
        });
        assert_eq!(result.diagnostics.output_annotations, 5);
        assert_eq!(result.diagnostics.annotation_rejections, 0);
        let t = result.transform;
        let center = t.position_from_point([10.0; 2]);
        let expected_x = egui::Rect::from_two_pos(
            t.position_from_point([1.0; 2]),
            t.position_from_point([10.0, 100.0]),
        );
        let expected_y = egui::Rect::from_two_pos(
            t.position_from_point([1.0, 10.0]),
            t.position_from_point([100.0; 2]),
        );
        for (fill, expected) in [(x_fill, expected_x), (y_fill, expected_y)] {
            let shape = out
                .shapes
                .iter()
                .find(|s| matches!(&s.shape, Shape::Rect(r) if r.fill == fill))
                .unwrap();
            let Shape::Rect(rect) = &shape.shape else {
                unreachable!()
            };
            assert_eq!(rect.rect, expected);
            assert_eq!(shape.clip_rect, t.rect());
        }
        assert!(out.shapes.iter().any(|s| matches!(&s.shape, Shape::LineSegment { points, .. } if points[0].x == center.x && points[1].x == center.x)));
        assert!(out.shapes.iter().any(|s| matches!(&s.shape, Shape::LineSegment { points, .. } if points[0].y == center.y && points[1].y == center.y)));
        out.drop_without_applying_deltas();
    }
}

#[test]
fn invalid_annotations_are_rejected_while_empty_and_offscreen_ones_are_skipped() {
    let ctx = Context::default();
    let mut state = PlotState::default();
    let annotations = vec![
        Annotation::text([f64::NAN, 1.0], "nan"),
        Annotation::text([0.0, 1.0], "log invalid"),
        Annotation::x_line(f64::INFINITY, "inf"),
        Annotation::y_line(-1.0, "log invalid"),
        Annotation::x_region([f64::NAN, 2.0], Color32::RED),
        Annotation::y_region([2.0, 1.0], Color32::RED),
        Annotation::XLine {
            value: 1.0,
            label: "bad width".into(),
            stroke: Some(Stroke::new(f32::NAN, Color32::RED)),
        },
        Annotation::YLine {
            value: 1.0,
            label: "negative width".into(),
            stroke: Some(Stroke::new(-1.0, Color32::RED)),
        },
        Annotation::text([f64::MAX; 2], "outside"),
        Annotation::text([1.0; 2], ""),
        Annotation::Text {
            point: [1.0; 2],
            text: "transparent".into(),
            color: Some(Color32::TRANSPARENT),
            anchor: Align2::CENTER_CENTER,
        },
        Annotation::XLine {
            value: 1.0,
            label: "zero width".into(),
            stroke: Some(Stroke::NONE),
        },
        Annotation::x_region([1.0; 2], Color32::RED),
        Annotation::y_region([f64::NEG_INFINITY, 0.0], Color32::RED),
        Annotation::y_region([0.0, f64::INFINITY], Color32::TRANSPARENT),
    ];
    let annotations: Arc<[Annotation]> = annotations.into();
    let out = run(&ctx, vec![], false, |ui| {
        Plot::new("p")
            .x_axis(Axis::new().scale(AxisScale::Log10))
            .y_axis(Axis::new().scale(AxisScale::Log10))
            .annotations(annotations.clone())
            .show(ui, &mut state, &[])
    });
    assert_eq!(out.diagnostics.annotation_rejections, 8);
    assert_eq!(out.diagnostics.output_annotations, 0);
}

#[test]
fn annotations_respect_parent_clip_submission_order_and_hidden_widgets() {
    let ctx = Context::default();
    let data = PlotData::new(vec![[0.0; 2], [1.0; 2]]);
    let fill = Color32::from_rgb(123, 45, 67);
    let annotations: Arc<[Annotation]> = vec![
        Annotation::x_region([f64::NEG_INFINITY, f64::INFINITY], fill),
        Annotation::text([0.5; 2], "ordered"),
    ]
    .into();
    let mut state = PlotState::default();
    let (result, out) = capture(&ctx, vec![], false, |ui| {
        ui.set_clip_rect(egui::Rect::from_min_max(
            egui::pos2(100.0, 100.0),
            egui::pos2(300.0, 250.0),
        ));
        Plot::new("p")
            .annotations(annotations.clone())
            .show(ui, &mut state, &[Line::new(&data)])
    });
    let series_index = out
        .shapes
        .iter()
        .position(|s| matches!(s.shape, Shape::Path(_)))
        .unwrap();
    let annotation_index = out
        .shapes
        .iter()
        .position(|s| matches!(&s.shape, Shape::Rect(r) if r.fill == fill))
        .unwrap();
    assert!(annotation_index > series_index);
    assert_eq!(
        out.shapes[annotation_index].clip_rect,
        result.transform.rect().intersect(egui::Rect::from_min_max(
            egui::pos2(100.0, 100.0),
            egui::pos2(300.0, 250.0)
        ))
    );
    out.drop_without_applying_deltas();
    let result = run(&ctx, vec![], false, |ui| {
        ui.set_clip_rect(egui::Rect::NOTHING);
        Plot::new("p")
            .annotations(annotations.clone())
            .show(ui, &mut state, &[Line::new(&data)])
    });
    assert!(result.diagnostics.painting_skipped);
    assert_eq!(result.diagnostics.output_annotations, 0);
}
