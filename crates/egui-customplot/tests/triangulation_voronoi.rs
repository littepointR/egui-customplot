mod support;

use std::f64::consts::TAU;

use egui::{Color32, Context, Event, Stroke};
use egui_customplot::{
    AlphaShape, AlphaShapeData, ColorMap, ColorScale, LinePattern, Plot, PlotBounds, PlotState,
    RegionKind, TriMesh2D, TriMeshError, TriPlot, Voronoi, VoronoiData,
};
use support::run;

#[test]
fn triplot_and_trimesh_dedup_edges_render_scalars_and_pick_vertices() {
    let sites = vec![
        [0.0, 0.0],
        [4.0, 0.0],
        [4.0, 3.0],
        [0.0, 3.0],
        [2.0, 1.5],
        [1.1, 2.2],
        [2.8, 0.9],
        [2.0, 1.5], // duplicate site must deduplicate cleanly
    ];
    let mesh = TriMesh2D::delaunay(&sites).unwrap();
    assert_eq!(mesh.vertices().len(), 7);
    // Undirected edges must all be unique with u < v.
    for w in mesh.edges().windows(2) {
        assert!(w[0] < w[1]);
    }
    for &[u, v] in mesh.edges() {
        assert!(u < v);
    }

    let collinear = [[0.0, 0.0], [1.0, 1.0], [2.0, 2.0]];
    assert_eq!(
        TriMesh2D::delaunay(&collinear).unwrap_err(),
        TriMeshError::DegenerateSites
    );

    let scalars: Vec<f64> = mesh.vertices().iter().map(|p| p[0] + p[1]).collect();
    let scale = ColorScale::new([0.0, 7.0])
        .unwrap()
        .color_map(ColorMap::Viridis);

    let ctx = Context::default();
    let mut state = PlotState::default();
    state.set_bounds(PlotBounds::new([-0.5, -0.5], [4.5, 3.5]).unwrap());
    let mut center_screen = None;
    let first = run(&ctx, vec![], false, |ui| {
        let r = Plot::new("trimesh").show_layers(
            ui,
            &mut state,
            &[TriPlot::new(&mesh)
                .markers(true)
                .face_fill(Color32::from_rgba_unmultiplied(80, 150, 220, 40))
                .vertex_scalars(&scalars, scale)
                .into()],
        );
        center_screen = Some(r.transform.position_from_point([2.0, 1.5]));
        r
    });
    assert!(first.diagnostics.geometry.output_paths >= mesh.edges().len());
    assert!(first.diagnostics.geometry.output_markers >= mesh.vertices().len());
    assert!(first.diagnostics.geometry.output_meshes >= 1);

    let hovered = run(
        &ctx,
        vec![Event::PointerMoved(center_screen.unwrap())],
        true,
        |ui| {
            Plot::new("trimesh").show_layers(
                ui,
                &mut state,
                &[TriPlot::new(&mesh).markers(true).into()],
            )
        },
    );
    let hit = hovered.hovered_point.expect("expected vertex hover hit");
    assert_eq!(hit.point, [2.0, 1.5]);
}

#[test]
fn voronoi_clips_rays_handles_collinear_and_picks_sites() {
    let bounds = PlotBounds::new([0.0, 0.0], [10.0, 10.0]).unwrap();
    let sites = vec![
        [2.0, 2.0],
        [8.0, 2.5],
        [7.5, 8.0],
        [2.5, 7.5],
        [5.0, 5.0],
        [5.0, 5.0], // duplicate
    ];
    let vor = VoronoiData::from_sites(&sites, bounds).unwrap();
    assert_eq!(vor.duplicate_sites(), 1);
    assert_eq!(vor.sites().len(), 5);
    assert!(!vor.edges().is_empty());
    for &[a, b] in vor.edges() {
        for p in [a, b] {
            assert!(
                p[0] >= 0.0 - 1e-9
                    && p[0] <= 10.0 + 1e-9
                    && p[1] >= 0.0 - 1e-9
                    && p[1] <= 10.0 + 1e-9,
                "edge point escaped clip bounds: {p:?}"
            );
        }
    }

    let collinear = VoronoiData::from_sites(&[[1.0, 5.0], [5.0, 5.0], [9.0, 5.0]], bounds).unwrap();
    assert_eq!(collinear.edges().len(), 2);

    let ctx = Context::default();
    let mut state = PlotState::default();
    state.set_bounds(bounds);
    let first = run(&ctx, vec![], false, |ui| {
        Plot::new("voronoi").show_layers(
            ui,
            &mut state,
            &[Voronoi::new(&vor)
                .pattern(LinePattern::Dashed {
                    length: 6.0,
                    gap: 3.0,
                })
                .into()],
        )
    });
    assert!(first.diagnostics.geometry.output_paths >= vor.edges().len());
    assert_eq!(first.diagnostics.geometry.output_markers, vor.sites().len());

    let site_pos = first.transform.position_from_point([5.0, 5.0]);
    let picked = run(&ctx, vec![Event::PointerMoved(site_pos)], true, |ui| {
        Plot::new("voronoi").show_layers(ui, &mut state, &[Voronoi::new(&vor).into()])
    });
    assert_eq!(picked.hovered_point.unwrap().point, [5.0, 5.0]);
}

#[test]
fn alphashape_hole_vs_critical_alpha_and_region_picking() {
    let mut ring = Vec::new();
    for &r in &[2.0_f64, 4.0_f64] {
        for k in 0..24 {
            let a = TAU * (k as f64 / 24.0);
            ring.push([5.0 + r * a.cos(), 5.0 + r * a.sin()]);
        }
    }
    let with_hole = AlphaShapeData::from_sites(&ring, 1.35).unwrap();
    assert!(with_hole.patch().face_count() > 0);
    // Inner hole boundary edges + outer boundary edges both appear cleanly.
    assert!(with_hole.boundary_edges().len() >= 24);

    let crit = AlphaShapeData::critical_alpha(&ring).unwrap();
    let filled = AlphaShapeData::from_sites(&ring, crit).unwrap();
    assert!(filled.patch().face_count() > with_hole.patch().face_count());

    let ctx = Context::default();
    let mut state = PlotState::default();
    state.set_bounds(PlotBounds::new([0.0, 0.0], [10.0, 10.0]).unwrap());
    let base = run(&ctx, vec![], false, |ui| {
        Plot::new("alpha").show_layers(
            ui,
            &mut state,
            &[AlphaShape::new(&with_hole)
                .fill_color(Color32::from_rgb(40, 140, 220))
                .border(Stroke::new(1.5, Color32::BLACK))
                .into()],
        )
    });
    assert!(base.diagnostics.geometry.output_meshes >= 1);
    assert!(base.diagnostics.geometry.output_paths >= 24);

    let hole_center = base.transform.position_from_point([5.0, 5.0]);
    let ring_pt = base.transform.position_from_point([8.0, 5.0]);

    let miss_hole = run(&ctx, vec![Event::PointerMoved(hole_center)], false, |ui| {
        Plot::new("alpha").show_layers(ui, &mut state, &[AlphaShape::new(&with_hole).into()])
    });
    assert!(miss_hole.hovered_region.is_none());

    let hit_ring = run(&ctx, vec![Event::PointerMoved(ring_pt)], true, |ui| {
        Plot::new("alpha").show_layers(ui, &mut state, &[AlphaShape::new(&with_hole).into()])
    });
    let reg = hit_ring
        .hovered_region
        .expect("expected alphaShape hit on ring");
    assert_eq!(reg.kind, RegionKind::AlphaShape);
}
