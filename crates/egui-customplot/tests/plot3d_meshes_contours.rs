use egui::{Context, Event};
use egui_customplot::{
    Bar3D, Bar3DData, BivariateNormalization, Contour3D, Contour3DData, ContourLevels,
    Histogram2Data, Layer3D, MeshSurface3D, Plot3D, Plot3DResponse, Plot3DState, ScalarField2D,
    Surface3DData,
};

#[allow(dead_code)]
#[path = "support/mod.rs"]
mod support;

fn run_plot3d(
    ctx: &Context,
    events: Vec<Event>,
    multipass: bool,
    mut show: impl FnMut(&mut egui::Ui) -> Plot3DResponse,
) -> Plot3DResponse {
    let mut out = None;
    let mut passes = 0usize;
    ctx.run_ui(support::input(events), |ui| {
        out = Some(show(ui));
        if multipass && passes == 0 {
            ui.ctx().request_discard("test stage9b meshes contours");
        }
        passes += 1;
    })
    .drop_without_applying_deltas();
    if multipass {
        assert_eq!(passes, 2);
    }
    out.unwrap()
}

fn sample_dome(rows: usize, cols: usize) -> Surface3DData {
    Surface3DData::from_explicit_fn([-1.5, 1.5], [-1.5, 1.5], [rows, cols], 50_000, |x, y| {
        1.2 - 0.3 * (x * x + y * y)
    })
    .unwrap()
}

#[test]
fn contour3_extracts_exact_height_isolines_from_field_and_surface() {
    let x = [-1.0, 0.0, 1.0];
    let y = [-1.0, 0.0, 1.0];
    let mut vals = Vec::with_capacity(9);
    for &yi in &y {
        for &xi in &x {
            vals.push(0.5 * (xi * xi + yi * yi));
        }
    }
    let field = ScalarField2D::new(x, y, vals).unwrap();
    let lvls = ContourLevels::Explicit(vec![0.25, 0.75]);

    let c_field = Contour3DData::from_field(&field, lvls.clone()).unwrap();
    assert!(!c_field.lines().is_empty());
    for line in c_field.lines() {
        for &p in line.points.iter() {
            assert!((p[2] - line.level).abs() < 1e-9);
        }
    }

    let surf = Surface3DData::from_field(&field);
    let c_surf = Contour3DData::from_surface(&surf, lvls).unwrap();
    assert!(!c_surf.lines().is_empty());
    for line in c_surf.lines() {
        for &p in line.points.iter() {
            assert!((p[2] - line.level).abs() < 1e-9);
        }
    }

    let ctx = Context::default();
    let mut st = Plot3DState::default();
    let resp = run_plot3d(&ctx, vec![], true, |ui| {
        Plot3D::new("contour3_test").height(260.0).show(
            ui,
            &mut st,
            &[Layer3D::from(Contour3D::new(&c_surf).name("c3"))],
        )
    });
    assert!(resp.diagnostics.plot3d.output_segments >= 4);

    let pt0 = c_surf.lines()[0].points[0];
    let pos0 = resp.transform.project_data(pt0).unwrap().0;
    let hovered = run_plot3d(&ctx, vec![Event::PointerMoved(pos0)], false, |ui| {
        Plot3D::new("contour3_test").height(260.0).show(
            ui,
            &mut st,
            &[Layer3D::from(Contour3D::new(&c_surf).name("c3"))],
        )
    });
    assert!(hovered.hovered_point.is_some());
}

#[test]
fn mesh_meshc_meshz_waterfall_and_ezmesh_match_topology_expectations() {
    let dome = sample_dome(10, 10)
        .with_contours(ContourLevels::Count(5))
        .unwrap();
    let ctx = Context::default();

    let mut st_hidden = Plot3DState::default();
    let hidden = run_plot3d(&ctx, vec![], true, |ui| {
        Plot3D::new("mesh_hidden").height(260.0).show(
            ui,
            &mut st_hidden,
            &[Layer3D::from(
                MeshSurface3D::new(&dome).name("mesh").hidden_line(true),
            )],
        )
    });
    assert!(hidden.diagnostics.plot3d.output_triangles > 0);
    assert!(hidden.diagnostics.plot3d.output_segments > 0);

    let mut st_see = Plot3DState::default();
    let see_thru = run_plot3d(&ctx, vec![], false, |ui| {
        Plot3D::new("mesh_see").height(260.0).show(
            ui,
            &mut st_see,
            &[Layer3D::from(MeshSurface3D::new(&dome).hidden_line(false))],
        )
    });
    assert_eq!(see_thru.diagnostics.plot3d.output_triangles, 0);
    assert_eq!(
        see_thru.diagnostics.plot3d.output_segments,
        hidden.diagnostics.plot3d.output_segments
    );

    // ML-067 `meshc` adds floor contour lines.
    let mut st_c = Plot3DState::default();
    let meshc = run_plot3d(&ctx, vec![], false, |ui| {
        Plot3D::new("meshc").height(260.0).show(
            ui,
            &mut st_c,
            &[Layer3D::from(
                MeshSurface3D::new(&dome).floor_contours(true),
            )],
        )
    });
    assert!(meshc.diagnostics.plot3d.output_segments > hidden.diagnostics.plot3d.output_segments);

    // ML-068 `meshz` adds boundary curtain drops + occluder faces.
    let mut st_z = Plot3DState::default();
    let meshz = run_plot3d(&ctx, vec![], false, |ui| {
        Plot3D::new("meshz").height(260.0).show(
            ui,
            &mut st_z,
            &[Layer3D::from(MeshSurface3D::curtain(&dome).name("meshz"))],
        )
    });
    assert!(meshz.diagnostics.plot3d.output_segments > hidden.diagnostics.plot3d.output_segments);
    assert!(meshz.diagnostics.plot3d.output_triangles > hidden.diagnostics.plot3d.output_triangles);

    // ML-069 `waterfall` omits cross-column grid lines on a 10x10 surface.
    let mut st_w = Plot3DState::default();
    let wf = run_plot3d(&ctx, vec![], false, |ui| {
        Plot3D::new("waterfall").height(260.0).show(
            ui,
            &mut st_w,
            &[Layer3D::from(
                MeshSurface3D::waterfall(&dome).name("waterfall"),
            )],
        )
    });
    assert!(wf.diagnostics.plot3d.output_segments > 0);
    assert!(wf.diagnostics.plot3d.output_segments < hidden.diagnostics.plot3d.output_segments);

    // ML-070 `fmesh` / LG-013 `ezmesh` / LG-014 `ezmeshc` + ML-022 `histogram2` 3D bars.
    let ez = Surface3DData::ez_explicit(10_000, |x, y| (0.5 * x).cos() * (0.5 * y).sin())
        .unwrap()
        .with_contours(ContourLevels::Count(4))
        .unwrap();
    let xs = [-1.0, -0.5, 0.2, 0.8, 1.1, -0.2, 0.4, 0.9, 1.2, -0.8];
    let ys = [-0.9, 0.1, 0.5, 1.0, -0.3, 0.2, -0.7, 0.6, 0.9, 0.8];
    let h2 = Histogram2Data::new(&xs, &ys, 4, 4).unwrap();
    let bars3d = Bar3DData::from_histogram2(&h2, BivariateNormalization::Count).unwrap();

    let mut st_mix = Plot3DState::default();
    let mix = run_plot3d(&ctx, vec![], false, |ui| {
        Plot3D::new("ezmesh_hist2").height(260.0).show(
            ui,
            &mut st_mix,
            &[
                Layer3D::from(MeshSurface3D::new(&ez).floor_contours(true).name("ezmeshc")),
                Layer3D::from(Bar3D::new(&bars3d).name("hist2_3d")),
            ],
        )
    });
    assert!(mix.diagnostics.plot3d.output_triangles > 0);
    assert!(mix.diagnostics.plot3d.output_segments > 0);
}
