use egui::{Color32, Context, Event, Shape, Stroke};
use egui_customplot::{
    ColorMap, ColorScale, ContourLevels, FunctionContourError, Layer3D, LightSource3D, Plot3D,
    Plot3DError, Plot3DResponse, Plot3DState, Ribbon3D, Ribbon3DData, Surface3D, Surface3DData,
    SurfaceShading,
};

#[allow(dead_code)]
#[path = "support/mod.rs"]
mod support;

fn run_plot3d(
    ctx: &Context,
    events: Vec<Event>,
    multipass: bool,
    mut show: impl FnMut(&mut egui::Ui) -> Plot3DResponse,
) -> (Plot3DResponse, Vec<Color32>) {
    let mut out = None;
    let mut passes = 0usize;
    let full = ctx.run_ui(support::input(events), |ui| {
        out = Some(show(ui));
        if multipass && passes == 0 {
            ui.ctx().request_discard("test stage9b surfaces");
        }
        passes += 1;
    });
    if multipass {
        assert_eq!(passes, 2);
    }
    let mut colors = Vec::new();
    for clipped in &full.shapes {
        collect_mesh_colors(&clipped.shape, &mut colors);
    }
    full.drop_without_applying_deltas();
    (out.unwrap(), colors)
}

fn collect_mesh_colors(shape: &Shape, out: &mut Vec<Color32>) {
    match shape {
        Shape::Mesh(mesh) => out.extend(mesh.vertices.iter().map(|v| v.color)),
        Shape::Vec(shapes) => {
            for s in shapes {
                collect_mesh_colors(s, out);
            }
        }
        _ => {}
    }
}

#[test]
fn surf_validates_renders_shadings_nan_holes_colorbar_and_primitive() {
    assert_eq!(
        Surface3DData::from_grid(&[0.0], &[0.0, 1.0], &[1.0, 2.0]).unwrap_err(),
        Plot3DError::InvalidDimensions
    );
    assert_eq!(
        Surface3DData::from_grid(&[0.0, 1.0], &[0.0, 1.0], &[1.0, 2.0, 3.0]).unwrap_err(),
        Plot3DError::LengthMismatch
    );

    let x = [-1.0, 0.0, 1.0, 2.0];
    let y = [-1.0, 0.0, 1.0, 2.0];
    let mut z = Vec::with_capacity(16);
    for &yi in &y {
        for &xi in &x {
            z.push(0.4 * (xi * xi - yi * yi));
        }
    }
    let surf = Surface3DData::from_grid(&x, &y, &z).unwrap();
    assert_eq!(surf.rows(), 4);
    assert_eq!(surf.cols(), 4);

    let ctx = Context::default();
    let mut st_facet = Plot3DState::default();
    let (r_facet, _) = run_plot3d(&ctx, vec![], true, |ui| {
        Plot3D::new("surf_facet").height(260.0).show(
            ui,
            &mut st_facet,
            &[Layer3D::from(
                Surface3D::new(&surf)
                    .name("peaks")
                    .shading(SurfaceShading::Faceted),
            )],
        )
    });
    assert_eq!(r_facet.diagnostics.plot3d.output_triangles, 18);
    assert!(r_facet.diagnostics.plot3d.output_segments >= 24);
    assert_eq!(r_facet.legend_entries.len(), 1);

    let mut st_flat = Plot3DState::default();
    let (r_flat, _) = run_plot3d(&ctx, vec![], false, |ui| {
        Plot3D::new("surf_flat").height(260.0).show(
            ui,
            &mut st_flat,
            &[Layer3D::from(
                Surface3D::new(&surf)
                    .name("flat")
                    .shading(SurfaceShading::Flat),
            )],
        )
    });
    assert_eq!(r_flat.diagnostics.plot3d.output_triangles, 18);
    assert_eq!(r_flat.diagnostics.plot3d.output_segments, 0);

    let mut st_interp = Plot3DState::default();
    let cs = ColorScale::new([-2.0, 2.0]).unwrap();
    let (r_interp, _) = run_plot3d(&ctx, vec![], false, |ui| {
        Plot3D::new("surf_interp")
            .height(260.0)
            .color_bar(cs, "Z")
            .show(
                ui,
                &mut st_interp,
                &[Layer3D::from(
                    Surface3D::new(&surf)
                        .name("interp")
                        .color_scale(cs)
                        .shading(SurfaceShading::Interp),
                )],
            )
    });
    assert_eq!(r_interp.diagnostics.plot3d.output_triangles, 18);
    assert_eq!(r_interp.diagnostics.plot3d.output_segments, 0);
    assert!(
        r_interp.plot_area.width() + 20.0 < r_flat.plot_area.width(),
        "colorbar should reserve horizontal margin"
    );

    // Vertex hover point (front-most corner) and quad interior hover region.
    let v_pos = r_interp
        .transform
        .project_data(surf.point(0, 3).unwrap())
        .unwrap()
        .0;
    let (r_pt, _) = run_plot3d(&ctx, vec![Event::PointerMoved(v_pos)], false, |ui| {
        Plot3D::new("surf_interp")
            .height(260.0)
            .color_bar(cs, "Z")
            .show(
                ui,
                &mut st_interp,
                &[Layer3D::from(
                    Surface3D::new(&surf)
                        .name("interp")
                        .color_scale(cs)
                        .shading(SurfaceShading::Interp),
                )],
            )
    });
    assert!(r_pt.hovered_point.is_some());

    let reg_pos = r_interp.transform.project_data([0.5, 0.5, 0.0]).unwrap().0;
    let (r_reg, _) = run_plot3d(&ctx, vec![Event::PointerMoved(reg_pos)], false, |ui| {
        Plot3D::new("surf_interp")
            .height(260.0)
            .color_bar(cs, "Z")
            .show(
                ui,
                &mut st_interp,
                &[Layer3D::from(
                    Surface3D::new(&surf)
                        .name("interp")
                        .color_scale(cs)
                        .shading(SurfaceShading::Interp),
                )],
            )
    });
    assert!(r_reg.hovered_region.is_some());

    // NaN vertex drops its 4 neighboring quads without bridging.
    let mut z_nan = z;
    z_nan[5] = f64::NAN;
    let surf_nan = Surface3DData::from_grid(&x, &y, &z_nan).unwrap();
    assert_eq!(surf_nan.non_finite_points(), 1);
    let mut st_nan = Plot3DState::default();
    let (r_nan, _) = run_plot3d(&ctx, vec![], false, |ui| {
        Plot3D::new("surf_nan").height(260.0).show(
            ui,
            &mut st_nan,
            &[Layer3D::from(
                Surface3D::new(&surf_nan).shading(SurfaceShading::Flat),
            )],
        )
    });
    assert_eq!(r_nan.diagnostics.plot3d.output_triangles, 10);

    // Low-level `surface` primitive skips legend and hover picking.
    let mut st_prim = Plot3DState::default();
    let (r_prim, _) = run_plot3d(&ctx, vec![Event::PointerMoved(v_pos)], false, |ui| {
        Plot3D::new("surf_prim").height(260.0).show(
            ui,
            &mut st_prim,
            &[Layer3D::from(
                Surface3D::primitive(&surf).name("hidden_prim"),
            )],
        )
    });
    assert!(r_prim.legend_entries.is_empty());
    assert!(r_prim.hovered_point.is_none());
    assert!(r_prim.hovered_region.is_none());
}

#[test]
fn surfc_ezsurfc_and_surfl_react_to_contours_and_lighting() {
    assert!(matches!(
        Surface3DData::ez_explicit(3, |x, y| x + y),
        Err(FunctionContourError::BudgetExceeded { .. })
    ));

    let ez = Surface3DData::ez_explicit(20_000, |x, y| {
        let r = (x * x + y * y).sqrt() + 1e-6;
        r.sin() / r
    })
    .unwrap()
    .with_contours(ContourLevels::Count(6))
    .unwrap();

    let ctx = Context::default();
    let mut st0 = Plot3DState::default();
    let (no_floor, _) = run_plot3d(&ctx, vec![], false, |ui| {
        Plot3D::new("surfc_off").height(260.0).show(
            ui,
            &mut st0,
            &[Layer3D::from(Surface3D::new(&ez).floor_contours(false))],
        )
    });
    let mut st1 = Plot3DState::default();
    let (with_floor, _) = run_plot3d(&ctx, vec![], false, |ui| {
        Plot3D::new("surfc_on").height(260.0).show(
            ui,
            &mut st1,
            &[Layer3D::from(Surface3D::new(&ez).floor_contours(true))],
        )
    });
    assert!(
        with_floor.diagnostics.plot3d.output_segments > no_floor.diagnostics.plot3d.output_segments
    );

    // ML-061 `surfl` colormap lighting changes vertex RGB when light angle changes.
    let plane =
        Surface3DData::from_explicit_fn([-1.0, 1.0], [-1.0, 1.0], [5, 5], 100, |x, _| 0.8 * x)
            .unwrap();
    let g_scale = ColorScale::new([0.0, 1.0])
        .unwrap()
        .color_map(ColorMap::Gray);
    let light_a = LightSource3D {
        azimuth_deg: 0.0,
        elevation_deg: 5.0,
        ..Default::default()
    };
    let light_b = LightSource3D {
        azimuth_deg: 180.0,
        elevation_deg: 50.0,
        ..Default::default()
    };

    let mut st_la = Plot3DState::default();
    let (_, cols_a) = run_plot3d(&ctx, vec![], false, |ui| {
        Plot3D::new("surfl_a").height(240.0).show(
            ui,
            &mut st_la,
            &[Layer3D::from(
                Surface3D::new(&plane)
                    .shading(SurfaceShading::Flat)
                    .colormap_lighting(true)
                    .color_scale(g_scale)
                    .wireframe_stroke(None)
                    .light(Some(light_a)),
            )],
        )
    });
    let mut st_lb = Plot3DState::default();
    let (_, cols_b) = run_plot3d(&ctx, vec![], false, |ui| {
        Plot3D::new("surfl_b").height(240.0).show(
            ui,
            &mut st_lb,
            &[Layer3D::from(
                Surface3D::new(&plane)
                    .shading(SurfaceShading::Flat)
                    .colormap_lighting(true)
                    .color_scale(g_scale)
                    .wireframe_stroke(None)
                    .light(Some(light_b)),
            )],
        )
    });
    assert!(!cols_a.is_empty() && cols_a.len() == cols_b.len());
    assert_ne!(
        cols_a, cols_b,
        "surfl lighting angle change must alter shaded colors"
    );
}

#[test]
fn ribbon_and_surfnorm_emit_strips_normals_and_outward_quivers() {
    let rib = Ribbon3DData::new(
        &[&[0.2, 0.8, 1.1, 0.4], &[0.9, 0.3, 0.7, 1.3]],
        Some(&[0.0, 1.0, 2.0, 3.0]),
        0.55,
    )
    .unwrap();
    assert_eq!(rib.strips().len(), 2);

    let ctx = Context::default();
    let mut st_r = Plot3DState::default();
    let (r_rib, _) = run_plot3d(&ctx, vec![], true, |ui| {
        Plot3D::new("ribbon_test").height(260.0).show(
            ui,
            &mut st_r,
            &[Layer3D::from(
                Ribbon3D::new(&rib)
                    .name("ribbons")
                    .edge_stroke(Some(Stroke::NONE)),
            )],
        )
    });
    assert!(r_rib.diagnostics.plot3d.output_triangles >= 12);
    let mid_pt = rib.strips()[0].point(1, 0).unwrap();
    let mid_next = rib.strips()[0].point(2, 1).unwrap();
    let centroid = [
        0.5 * (mid_pt[0] + mid_next[0]),
        0.5 * (mid_pt[1] + mid_next[1]),
        0.5 * (mid_pt[2] + mid_next[2]),
    ];
    let pos = r_rib.transform.project_data(centroid).unwrap().0;
    let (r_pick, _) = run_plot3d(&ctx, vec![Event::PointerMoved(pos)], false, |ui| {
        Plot3D::new("ribbon_test").height(260.0).show(
            ui,
            &mut st_r,
            &[Layer3D::from(Ribbon3D::new(&rib).name("ribbons"))],
        )
    });
    assert!(
        r_pick
            .hovered_region
            .as_ref()
            .is_some_and(|reg| reg.label.contains("ribbon"))
    );

    // EX-007 `surfnorm` on unit sphere yields outward normals away from poles.
    let sph = Surface3DData::sphere(12).unwrap();
    let nq = sph.normals_quiver(2, 0.25);
    assert!(!nq.arrows().is_empty());
    for arr in nq.arrows() {
        let p = arr.origin;
        if p[2].abs() < 0.95 {
            let dot = p[0] * arr.vector[0] + p[1] * arr.vector[1] + p[2] * arr.vector[2];
            assert!(
                dot > 0.24,
                "expected outward scaled sphere normal at {p:?}, got {dot}"
            );
        }
    }

    let mut st_s0 = Plot3DState::default();
    let (base, _) = run_plot3d(&ctx, vec![], false, |ui| {
        Plot3D::new("sn0").height(250.0).show(
            ui,
            &mut st_s0,
            &[Layer3D::from(
                Surface3D::new(&sph).shading(SurfaceShading::Flat),
            )],
        )
    });
    let mut st_s1 = Plot3DState::default();
    let (with_n, _) = run_plot3d(&ctx, vec![], false, |ui| {
        Plot3D::new("sn1").height(250.0).show(
            ui,
            &mut st_s1,
            &[Layer3D::from(
                Surface3D::new(&sph)
                    .shading(SurfaceShading::Flat)
                    .show_normals(true, 0.25)
                    .normal_stride(2),
            )],
        )
    });
    assert!(with_n.diagnostics.plot3d.output_segments > base.diagnostics.plot3d.output_segments);
    assert!(with_n.diagnostics.plot3d.output_triangles > base.diagnostics.plot3d.output_triangles);
}
