mod support;

use std::time::Duration;

use egui::{Color32, Context, Event, Shape, Stroke};
use egui_customplot::{
    AnimatedLine, AnimatedLine2D, Annotation, AnnotationCoord, Comet, CometPlayback, Figure, Plot,
    PlotBounds, PlotData, PlotState,
};
use support::{capture, run};

#[test]
fn animated_line_multipass_pause_clear_and_idle_repaint() {
    let ctx = Context::default();
    let mut anim = AnimatedLine2D::new(4, 4 * 16).unwrap();
    let mut seen_rev = anim.revision();

    // First batch advances revision once even across discarded layout pass
    for pass in 0..2 {
        let rep = anim
            .add_points_once(1, &[[0.0, 0.0], [1.0, 1.0], [2.0, 0.0]])
            .unwrap();
        assert_eq!(rep.applied, pass == 0);
    }
    assert_eq!(anim.len(), 3);
    assert!(anim.sync_repaint(&ctx, &mut seen_rev));
    assert!(!anim.sync_repaint(&ctx, &mut seen_rev)); // EGUI-08: zero spurious repaint

    // Ring eviction keeps newest 4
    let r2 = anim.add_points_once(2, &[[3.0, 2.0], [4.0, 3.0]]).unwrap();
    assert!(r2.applied);
    assert_eq!(anim.len(), 4);
    assert_eq!(anim.data().points()[0], [1.0, 1.0]);

    // Pause blocks subsequent batch applications & suppresses repaint
    anim.set_paused(true);
    let r3 = anim.add_points_once(3, &[[5.0, 4.0]]).unwrap();
    assert!(!r3.applied);
    assert!(!anim.sync_repaint(&ctx, &mut seen_rev));

    let mut state = PlotState::default();
    let resp = run(&ctx, vec![], true, |ui| {
        Plot::new("anim").show_layers(ui, &mut state, &[AnimatedLine::new(&anim).into()])
    });
    assert_eq!(resp.diagnostics.input_points, 4);
    assert_eq!(resp.diagnostics.output_paths, 1);

    anim.clear().unwrap();
    assert!(anim.is_empty());
}

#[test]
fn comet_frame_idempotency_time_consistency_repaint_and_picking() {
    let pts: Vec<[f64; 2]> = (0..11)
        .map(|i| [i as f64, (i as f64 * 0.5).sin()])
        .collect();
    let data = PlotData::new(pts.clone());

    let mut p_split = CometPlayback::new(4.0).tail_fraction(0.3);
    let mut p_single = CometPlayback::new(4.0).tail_fraction(0.3);
    // Repeated frame_id=10 in multipass only advances once
    assert!(p_split.advance_once(10, 0.5, data.len()));
    assert!(!p_split.advance_once(10, 0.5, data.len()));
    assert!(p_split.advance_once(11, 0.5, data.len()));
    assert!(p_single.advance_once(10, 1.0, data.len()));
    assert!((p_split.progress_points() - p_single.progress_points()).abs() < 1e-12);
    assert!((p_split.progress_points() - 4.0).abs() < 1e-12);

    let ctx = Context::default();
    assert!(p_split.sync_repaint(&ctx, data.len(), Duration::from_millis(16)));
    // Advancing past the end stops non-looping playback and ceases repaint scheduling (EGUI-08)
    assert!(p_split.advance_once(12, 10.0, data.len()));
    assert!(!p_split.is_playing());
    assert!(!p_split.sync_repaint(&ctx, data.len(), Duration::from_millis(16)));

    p_split.reset();
    p_split.set_progress_points(5.5, data.len());
    let mut state = PlotState::default();
    state.set_bounds(PlotBounds::new([0.0, -1.5], [10.0, 1.5]).unwrap());
    let first = run(&ctx, vec![], false, |ui| {
        Plot::new("comet").show_layers(
            ui,
            &mut state,
            &[Comet::new(&data, &p_split).head_radius(6.0).into()],
        )
    });
    assert_eq!(first.diagnostics.flow.output_heads, 1);
    assert!(first.diagnostics.flow.output_paths >= 3);

    let expected_head = [5.5, 0.5 * (pts[5][1] + pts[6][1])];
    let head_pos = first.transform.position_from_point(expected_head);
    let picked = run(&ctx, vec![Event::PointerMoved(head_pos)], true, |ui| {
        Plot::new("comet").show_layers(ui, &mut state, &[Comet::new(&data, &p_split).into()])
    });
    let hit = picked.hovered_point.expect("expected comet head pick");
    assert!((hit.point[0] - expected_head[0]).abs() < 1e-12);
}

#[test]
fn ex027_annotations_and_figure_annotations_render_without_affecting_bounds() {
    let data = PlotData::new(vec![[1.0, 2.0], [5.0, 6.0]]);
    let plot_annots = vec![
        Annotation::line(AnnotationCoord::Normalized, [0.1, 0.1], [0.9, 0.9])
            .with_stroke(Stroke::new(1.5, Color32::RED)),
        Annotation::arrow(AnnotationCoord::Data, [1.0, 5.0], [4.0, 5.0])
            .with_stroke(Stroke::new(1.5, Color32::BLUE))
            .with_head_size(10.0, 8.0),
        Annotation::double_arrow(AnnotationCoord::Normalized, [0.2, 0.8], [0.8, 0.8])
            .with_stroke(Stroke::new(1.5, Color32::GREEN)),
        Annotation::text_arrow(AnnotationCoord::Data, [2.0, 3.0], [4.0, 4.0], "peak")
            .with_stroke(Stroke::new(1.5, Color32::BLACK))
            .with_color(Color32::BLACK),
        Annotation::textbox(
            AnnotationCoord::Normalized,
            [[0.1, 0.7], [0.35, 0.9]],
            "box",
        )
        .with_fill(Color32::from_rgba_unmultiplied(240, 240, 200, 180))
        .with_stroke(Stroke::new(1.0, Color32::DARK_GRAY)),
        Annotation::rectangle(AnnotationCoord::Data, [[2.0, 2.5], [3.5, 4.5]])
            .with_fill(Color32::from_rgba_unmultiplied(50, 120, 200, 50))
            .with_stroke(Stroke::new(1.0, Color32::BLUE)),
        Annotation::ellipse(AnnotationCoord::Normalized, [[0.55, 0.2], [0.85, 0.5]])
            .with_fill(Color32::from_rgba_unmultiplied(200, 90, 40, 60))
            .with_stroke(Stroke::new(1.5, Color32::BROWN)),
        // Out-of-range Normalized annotation must reject instead of painting garbage
        Annotation::line(AnnotationCoord::Normalized, [f64::NAN, 0.0], [1.0, 1.0]),
    ];

    let ctx = Context::default();
    let mut s_plain = PlotState::default();
    let plain = run(&ctx, vec![], false, |ui| {
        Plot::new("plain").show(ui, &mut s_plain, &[egui_customplot::Line::new(&data)])
    });

    let mut s_ann = PlotState::default();
    let (annotated, out) = capture(&ctx, vec![], false, |ui| {
        Plot::new("ann").annotations(plot_annots.as_slice()).show(
            ui,
            &mut s_ann,
            &[egui_customplot::Line::new(&data)],
        )
    });
    let path_count = out
        .shapes
        .iter()
        .filter(|s| {
            matches!(
                s.shape,
                Shape::Path(_) | Shape::LineSegment { .. } | Shape::Rect(_)
            )
        })
        .count();
    out.drop_without_applying_deltas();

    assert_eq!(plain.transform.bounds(), annotated.transform.bounds());
    assert_eq!(annotated.diagnostics.output_annotations, 7);
    assert_eq!(annotated.diagnostics.annotation_rejections, 1);
    assert!(path_count > 7);

    // Figure-level Normalized annotations succeed while Figure-level Data annotation is rejected
    let fig_annots = vec![
        Annotation::line(AnnotationCoord::Normalized, [0.05, 0.5], [0.95, 0.5])
            .with_stroke(Stroke::new(1.5, Color32::RED)),
        Annotation::rectangle(AnnotationCoord::Normalized, [[0.1, 0.1], [0.3, 0.3]])
            .with_stroke(Stroke::new(1.0, Color32::BLUE)),
        Annotation::arrow(AnnotationCoord::Data, [0.0, 0.0], [1.0, 1.0]),
    ];
    let mut cell_state = PlotState::default();
    ctx.run_ui(support::input(vec![]), |ui| {
        let fig = Figure::new("fig-ann")
            .columns(2)
            .annotations(fig_annots.clone())
            .show(ui, 2, |cell| {
                cell.plot(Plot::new("fig-cell"))
                    .show(cell.ui(), &mut cell_state, &[])
            })
            .unwrap();
        assert_eq!(fig.output_annotations, 2);
        assert_eq!(fig.annotation_rejections, 1);
    })
    .drop_without_applying_deltas();
}
