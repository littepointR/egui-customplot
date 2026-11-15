mod support;

use egui::{Color32, Context, Event, IdSalt};
use egui_customplot::{
    Line, MarkerShape, MarkerStyle, Plot, PlotBounds, PlotData, PlotState, Scatter, Series,
};
use support::{input, run};

#[test]
fn mixed_series_share_samples_and_batch_markers_in_one_mesh_per_series() {
    let ctx = Context::default();
    let data = PlotData::new(
        (0..100)
            .map(|i| [i as f64, (i as f64).sin()])
            .collect::<Vec<_>>(),
    );
    let colors = vec![Color32::YELLOW; data.len()];
    let radii = vec![4.0; data.len()];
    let series = [
        Line::new(&data).markers(MarkerStyle::default()).into(),
        Scatter::new(&data)
            .colors(&colors)
            .unwrap()
            .radii(&radii)
            .unwrap()
            .into(),
    ];
    let mut state = PlotState::default();
    let mut result = None;
    let out = ctx.run_ui(input(vec![]), |ui| {
        result = Some(Plot::new("mixed").show_series(ui, &mut state, &series));
    });
    let result = result.unwrap();
    assert_eq!(result.diagnostics.unique_sample_bytes, data.sample_bytes());
    assert_eq!(result.diagnostics.input_points, 200);
    assert_eq!(result.diagnostics.output_markers, 200);
    assert_eq!(result.diagnostics.visited_segments, 99);
    assert_eq!(result.diagnostics.visited_marker_points, 200);
    let meshes: Vec<_> = out
        .shapes
        .iter()
        .filter_map(|shape| {
            if let egui::Shape::Mesh(mesh) = &shape.shape {
                Some(mesh)
            } else {
                None
            }
        })
        .collect();
    assert_eq!(meshes.len(), 2, "not one egui Shape per marker");
    for mesh in meshes {
        assert!(mesh.is_valid());
        assert!(mesh.vertices.iter().all(|v| v.pos.is_finite()));
    }
    out.drop_without_applying_deltas();
}

#[test]
fn hover_reports_original_sample_and_skips_transparent_zero_and_hidden_points() {
    let ctx = Context::default();
    let data = PlotData::new(vec![[1.0; 2], [2.0; 2], [3.0; 2], [4.0; 2]]);
    let colors = [
        Color32::RED,
        Color32::TRANSPARENT,
        Color32::BLUE,
        Color32::GREEN,
    ];
    let radii = [4.0, 4.0, 0.0, 4.0];
    let series = [Scatter::new(&data)
        .id_salt("samples")
        .colors(&colors)
        .unwrap()
        .radii(&radii)
        .unwrap()
        .into()];
    let mut state = PlotState::default();
    let initial = run(&ctx, vec![], false, |ui| {
        Plot::new("p").show_series(ui, &mut state, &series)
    });
    for index in 0..4 {
        let pos = initial.transform.position_from_point(data.points()[index]);
        let out = run(&ctx, vec![Event::PointerMoved(pos)], false, |ui| {
            Plot::new("p").show_series(ui, &mut state, &series)
        });
        assert_eq!(out.diagnostics.visited_pick_points, 4);
        if index == 0 || index == 3 {
            let point = out.hovered_point.unwrap();
            assert_eq!(point.point_index, index);
            assert_eq!(point.point, data.points()[index]);
            assert_eq!(point.series_id, IdSalt::new("samples"));
            assert!(point.screen_distance < 0.001);
        } else {
            assert!(out.hovered_point.is_none());
        }
    }
    state.set_series_visible(IdSalt::new("samples"), false);
    let out = run(&ctx, vec![], false, |ui| {
        Plot::new("p").show_series(ui, &mut state, &series)
    });
    assert_eq!(out.diagnostics.visible_series, 0);
    assert_eq!(out.diagnostics.visited_pick_points, 0);
    assert_eq!(out.diagnostics.output_markers, 0);
    assert!(out.hovered_point.is_none());
}

#[test]
fn invalid_scalar_marker_radius_is_neither_rendered_nor_picked() {
    for radius in [f32::INFINITY, f32::NAN, -1.0, 0.0] {
        let ctx = Context::default();
        let data = PlotData::new(vec![[1.0; 2]]);
        let series = [Scatter::new(&data).radius(radius).into()];
        let mut state = PlotState::default();
        let first = run(&ctx, vec![], false, |ui| {
            Plot::new("p").show_series(ui, &mut state, &series)
        });
        let pos = first.transform.position_from_point([1.0; 2]);
        let out = run(&ctx, vec![Event::PointerMoved(pos)], false, |ui| {
            Plot::new("p").show_series(ui, &mut state, &series)
        });
        assert_eq!(out.diagnostics.output_markers, 0);
        assert_eq!(
            out.diagnostics.numeric_rejections,
            usize::from(radius != 0.0)
        );
        assert!(out.hovered_point.is_none());
    }
}

#[test]
fn hidden_series_do_not_affect_auto_bounds_or_scan_geometry() {
    let ctx = Context::default();
    let near = PlotData::new(vec![[1.0; 2], [2.0; 2]]);
    let far = PlotData::new(vec![[1000.0; 2], [2000.0; 2]]);
    let series: [Series<'_>; 2] = [
        Line::new(&near).id_salt("near").into(),
        Scatter::new(&far).id_salt("far").into(),
    ];
    let mut state = PlotState::default();
    state.set_series_visible(IdSalt::new("far"), false);
    let out = run(&ctx, vec![], false, |ui| {
        Plot::new("p").show_series(ui, &mut state, &series)
    });
    assert!(out.transform.bounds().max()[0] < 3.0);
    assert_eq!(out.diagnostics.visible_series, 1);
    assert_eq!(out.diagnostics.visited_segments, 1);
    assert_eq!(out.diagnostics.visited_marker_points, 0);
    assert_eq!(
        out.diagnostics.unique_sample_bytes,
        near.sample_bytes() + far.sample_bytes()
    );
}

#[test]
fn all_marker_shapes_support_singletons_and_hidpi() {
    let ctx = Context::default();
    let data = PlotData::new(vec![[0.5; 2]]);
    for ppp in [1.0, 2.0] {
        ctx.set_pixels_per_point(ppp);
        for shape in [
            MarkerShape::Circle,
            MarkerShape::Square,
            MarkerShape::Diamond,
            MarkerShape::TriangleUp,
            MarkerShape::TriangleDown,
            MarkerShape::Plus,
            MarkerShape::Cross,
        ] {
            let mut state = PlotState::default();
            state.set_bounds(PlotBounds::new([0.0; 2], [1.0; 2]).unwrap());
            let out = run(&ctx, vec![], false, |ui| {
                Plot::new("shape").show_series(
                    ui,
                    &mut state,
                    &[Scatter::new(&data).shape(shape).into()],
                )
            });
            assert_eq!(out.diagnostics.output_markers, 1);
            assert_eq!(out.diagnostics.numeric_rejections, 0);
            assert!(out.diagnostics.geometry_capacity_bytes > 0);
        }
    }
}

#[test]
fn hover_disabled_avoids_linear_pick_scan() {
    let ctx = Context::default();
    let data = PlotData::new(vec![[1.0; 2], [2.0; 2]]);
    let mut state = PlotState::default();
    let first = run(&ctx, vec![], false, |ui| {
        Plot::new("p").show(ui, &mut state, &[Line::new(&data)])
    });
    let out = run(
        &ctx,
        vec![Event::PointerMoved(first.transform.rect().center())],
        false,
        |ui| {
            Plot::new("p")
                .show_hover(false)
                .show(ui, &mut state, &[Line::new(&data)])
        },
    );
    assert_eq!(out.diagnostics.visited_pick_points, 0);
    assert!(out.hovered_point.is_none());
}
