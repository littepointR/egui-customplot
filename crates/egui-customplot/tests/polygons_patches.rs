mod support;

use egui::{Color32, Context, Event, Shape, Stroke};
use egui_customplot::{
    ColorMap, ColorScale, DataRect, Fill, Line, Patch, PatchData, Plot, PlotBounds, PlotData,
    PlotState, PolyRegion, PolyShape, PolyShapeData, PolygonData, RectData, Rectangles, RegionKind,
    SeriesError,
};
use support::{capture, run};

#[test]
fn line_primitive_composes_additively_and_reflects_host_data_replacement() {
    let ctx = Context::default();
    let a = PlotData::new(vec![[0.0, 0.0], [1.0, 1.0]]);
    let b = PlotData::new(vec![[0.0, 1.0], [1.0, 0.0]]);
    let mut state = PlotState::default();
    let (out, output) = capture(&ctx, vec![], false, |ui| {
        Plot::new("prim").show_layers(
            ui,
            &mut state,
            &[
                Line::primitive(&a).name("A").into(),
                Line::primitive(&b).name("B").into(),
            ],
        )
    });
    assert_eq!(out.diagnostics.output_paths, 2);
    output.drop_without_applying_deltas();

    let b2 = PlotData::new(vec![[0.0, 2.0], [1.0, 2.0], [2.0, 2.0]]);
    let out = run(&ctx, vec![], false, |ui| {
        Plot::new("prim").show_layers(
            ui,
            &mut state,
            &[Line::primitive(&a).into(), Line::primitive(&b2).into()],
        )
    });
    assert_eq!(out.diagnostics.input_points, 5);
}

#[test]
fn concave_fill_and_confidence_band_skip_notch_and_split_nan_gaps() {
    let ctx = Context::default();
    let concave = PolygonData::new(vec![
        [0.0, 0.0],
        [6.0, 0.0],
        [6.0, 6.0],
        [4.0, 6.0],
        [4.0, 2.0],
        [2.0, 2.0],
        [2.0, 6.0],
        [0.0, 6.0],
    ]);
    let mut state = PlotState::default();
    let (out, output) = capture(&ctx, vec![], false, |ui| {
        Plot::new("fill").show_layers(
            ui,
            &mut state,
            &[Fill::new(&concave).color(Color32::BLUE).into()],
        )
    });
    assert!(out.diagnostics.patches.output_triangles >= 6);
    output.drop_without_applying_deltas();

    let notch = out.transform.position_from_point([3.0, 4.0]);
    let arm = out.transform.position_from_point([1.0, 4.0]);
    let miss = run(&ctx, vec![Event::PointerMoved(notch)], false, |ui| {
        Plot::new("fill").show_layers(ui, &mut state, &[Fill::new(&concave).into()])
    });
    assert!(miss.hovered_region.is_none());
    let hit = run(&ctx, vec![Event::PointerMoved(arm)], false, |ui| {
        Plot::new("fill").show_layers(ui, &mut state, &[Fill::new(&concave).into()])
    });
    assert_eq!(hit.hovered_region.unwrap().kind, RegionKind::Fill);

    let band = PolygonData::confidence_band(
        &[0.0, 1.0, 2.0, 3.0, 4.0],
        &[0.0, 0.0, f64::NAN, 1.0, 1.0],
        &[1.0, 1.0, 1.0, 2.0, 2.0],
    )
    .unwrap();
    assert_eq!(band.ring_count(), 2);
}

#[test]
fn polyshape_hole_does_not_capture_hover_while_outer_ring_does() {
    let ctx = Context::default();
    let region = PolyRegion::new(vec![[0.0, 0.0], [10.0, 0.0], [10.0, 10.0], [0.0, 10.0]])
        .hole(vec![[3.0, 3.0], [7.0, 3.0], [7.0, 7.0], [3.0, 7.0]]);
    let data = PolyShapeData::new(vec![region]);
    let mut state = PlotState::default();
    let initial = run(&ctx, vec![], false, |ui| {
        Plot::new("ps").show_layers(
            ui,
            &mut state,
            &[PolyShape::new(&data)
                .color(Color32::GOLD)
                .border(Stroke::new(1.0, Color32::BLACK))
                .into()],
        )
    });
    assert!(initial.diagnostics.patches.output_triangles >= 8);
    let hole_pos = initial.transform.position_from_point([5.0, 5.0]);
    let ring_pos = initial.transform.position_from_point([1.5, 5.0]);
    let hole_hit = run(&ctx, vec![Event::PointerMoved(hole_pos)], false, |ui| {
        Plot::new("ps").show_layers(ui, &mut state, &[PolyShape::new(&data).into()])
    });
    assert!(hole_hit.hovered_region.is_none());
    let ring_hit = run(&ctx, vec![Event::PointerMoved(ring_pos)], false, |ui| {
        Plot::new("ps").show_layers(ui, &mut state, &[PolyShape::new(&data).into()])
    });
    assert_eq!(ring_hit.hovered_region.unwrap().kind, RegionKind::PolyShape);
}

#[test]
fn patch_supports_face_and_gouraud_coloring_and_validates_lengths() {
    let verts = vec![[0.0, 0.0], [2.0, 0.0], [2.0, 2.0], [0.0, 2.0], [3.0, 0.0]];
    let patch_data =
        PatchData::from_indexed(verts, vec![0, 4, 7], vec![0, 1, 2, 3, 1, 4, 2]).unwrap();
    let scale = ColorScale::new([0.0, 10.0])
        .unwrap()
        .color_map(ColorMap::Viridis);
    assert_eq!(
        Patch::new(&patch_data)
            .face_colors(&[Color32::RED])
            .unwrap_err(),
        SeriesError::LengthMismatch
    );
    assert_eq!(
        Patch::new(&patch_data)
            .vertex_scalars(&[0.0], scale)
            .unwrap_err(),
        SeriesError::LengthMismatch
    );

    let ctx = Context::default();
    let v_scalars = [0.0, 5.0, 10.0, 5.0, f64::NAN];
    let patch = Patch::new(&patch_data)
        .vertex_scalars(&v_scalars, scale)
        .unwrap();
    let mut state = PlotState::default();
    let (out, output) = capture(&ctx, vec![], false, |ui| {
        Plot::new("patch").show_layers(ui, &mut state, &[patch.into()])
    });
    // Face 0 (quad -> 2 tris) has all finite vertices; face 1 contains NaN vertex 4 and is skipped.
    assert_eq!(out.diagnostics.patches.output_triangles, 2);
    let mesh = output
        .shapes
        .iter()
        .find_map(|s| match &s.shape {
            Shape::Mesh(m) => Some(m),
            _ => None,
        })
        .unwrap();
    assert!(mesh.vertices.windows(2).any(|w| w[0].color != w[1].color));
    output.drop_without_applying_deltas();

    let center = out.transform.position_from_point([1.0, 1.0]);
    let nan_face = out.transform.position_from_point([2.4, 0.8]);
    let hit = run(&ctx, vec![Event::PointerMoved(center)], false, |ui| {
        Plot::new("patch").show_layers(ui, &mut state, &[patch.into()])
    });
    let r = hit.hovered_region.unwrap();
    assert_eq!(r.kind, RegionKind::Patch);
    assert_eq!(r.item_index, 0);
    assert!((r.scalar_value.unwrap() - 5.0).abs() < 0.25);

    let miss = run(&ctx, vec![Event::PointerMoved(nan_face)], false, |ui| {
        Plot::new("patch").show_layers(ui, &mut state, &[patch.into()])
    });
    assert!(miss.hovered_region.is_none());
}

#[test]
fn rectangles_and_ellipses_hit_test_rounded_geometry() {
    let ctx = Context::default();
    let data = RectData::new(vec![
        DataRect::new([0.0, 0.0], [2.0, 2.0]),
        DataRect::ellipse([5.0, 5.0], [2.0, 2.0]),
    ]);
    let mut state = PlotState::default();
    state.set_bounds(PlotBounds::new([-1.0, -1.0], [8.0, 8.0]).unwrap());
    let initial = run(&ctx, vec![], false, |ui| {
        Plot::new("rects").show_layers(ui, &mut state, &[Rectangles::new(&data).into()])
    });
    assert!(initial.diagnostics.patches.output_triangles >= 10);
    let corner_outside = initial.transform.position_from_point([3.1, 3.1]);
    let ellipse_center = initial.transform.position_from_point([5.0, 5.0]);
    let miss = run(
        &ctx,
        vec![Event::PointerMoved(corner_outside)],
        false,
        |ui| Plot::new("rects").show_layers(ui, &mut state, &[Rectangles::new(&data).into()]),
    );
    assert!(miss.hovered_region.is_none());
    let hit = run(
        &ctx,
        vec![Event::PointerMoved(ellipse_center)],
        false,
        |ui| Plot::new("rects").show_layers(ui, &mut state, &[Rectangles::new(&data).into()]),
    );
    let region = hit.hovered_region.unwrap();
    assert_eq!(region.kind, RegionKind::Rectangle);
    assert_eq!(region.item_index, 1);
}
