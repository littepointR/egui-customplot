use egui::{Context, Event};
use egui_customplot::{
    Bounds3D, ColorScale, DepthMode, IsocapThreshold, Isocaps3D, Isocaps3DData, Isosurface3D,
    Isosurface3DData, Layer3D, Plot3D, Plot3DResponse, Plot3DState, ScalarVolume3D, TetraMesh3D,
    TetraMesh3DData, TriMeshError,
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
            ui.ctx()
                .request_discard("test stage9c isosurface tetramesh");
        }
        passes += 1;
    })
    .drop_without_applying_deltas();
    if multipass {
        assert_eq!(passes, 2);
    }
    out.unwrap()
}

#[test]
fn isosurface_and_isocaps_align_at_volume_boundaries_and_color_from_secondary_field() {
    let bounds = Bounds3D::new([-0.7, -0.7, -0.7], [1.3, 1.3, 1.3]).unwrap();
    let vol = ScalarVolume3D::from_function(bounds, [20, 20, 20], 20_000, |x, y, z| {
        x * x + y * y + z * z
    })
    .unwrap();
    let color_vol =
        ScalarVolume3D::from_function(bounds, [12, 12, 12], 5_000, |x, y, z| x + 0.5 * y - z)
            .unwrap();

    let iso = Isosurface3DData::from_volume(&vol, 1.0)
        .unwrap()
        .with_colors_from_volume(&color_vol);
    let iso_mesh = iso.mesh().expect("non-empty sphere isosurface");
    assert!((iso.isovalue() - 1.0).abs() < 1e-12);

    // Secondary scalar variation across isosurface vertices.
    let s_range = iso_mesh.scalar_range().unwrap();
    assert!(s_range[1] > s_range[0] + 0.2);

    let caps = Isocaps3DData::from_volume(&vol, 1.0, IsocapThreshold::Below).unwrap();
    let caps_mesh = caps
        .mesh()
        .expect("open clipped unit sphere must have end caps");
    for (p, &s) in caps_mesh
        .vertices()
        .iter()
        .zip(caps_mesh.vertex_scalars().iter())
    {
        assert!(
            s <= 1.0005,
            "IsocapThreshold::Below vertex scalar > iso: {s}"
        );
        let on_box_boundary = (p[0] + 0.7).abs() < 1e-6
            || (p[0] - 1.3).abs() < 1e-6
            || (p[1] + 0.7).abs() < 1e-6
            || (p[1] - 1.3).abs() < 1e-6
            || (p[2] + 0.7).abs() < 1e-6
            || (p[2] - 1.3).abs() < 1e-6;
        assert!(on_box_boundary, "cap vertex not on boundary box: {p:?}");
    }

    let cscale = ColorScale::new([-2.0, 2.0]).unwrap();
    let ctx = Context::default();
    let mut state = Plot3DState::default();
    let resp = run_plot3d(&ctx, vec![], true, |ui| {
        Plot3D::new("iso_caps_test")
            .height(260.0)
            .depth_mode(DepthMode::ZBuffer)
            .show(
                ui,
                &mut state,
                &[
                    Layer3D::from(Isosurface3D::new(&iso).name("iso").color_scale(cscale)),
                    Layer3D::from(Isocaps3D::new(&caps).name("caps").color_scale(cscale)),
                ],
            )
    });
    assert_eq!(
        resp.diagnostics.plot3d.output_triangles,
        iso_mesh.triangles().len() + caps_mesh.triangles().len()
    );
    assert!(resp.diagnostics.plot3d.zbuffer_pixels > 0);
}

#[test]
fn tetramesh_culls_shared_interior_faces_and_picks_tetrahedra() {
    assert_eq!(
        TetraMesh3DData::new(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [0.0, 0.0, 1.0],
            ],
            vec![[0, 1, 2, 2]],
        )
        .unwrap_err(),
        TriMeshError::InvalidTriangleIndex
    );

    let verts = vec![
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        [0.0, 0.0, 1.0],
        [1.0, 1.0, 1.0],
    ];
    let tets = vec![[0, 1, 2, 3], [1, 2, 3, 4]];
    let tm = TetraMesh3DData::new(verts, tets)
        .unwrap()
        .with_cell_scalars(&[0.2, 0.9])
        .unwrap();
    assert_eq!(tm.all_faces().len(), 8);
    assert_eq!(tm.boundary_faces().len(), 6);
    assert_eq!(tm.edges().len(), 9);

    let ctx = Context::default();
    let mut st0 = Plot3DState::default();
    let r_bound = run_plot3d(&ctx, vec![], true, |ui| {
        Plot3D::new("tet_bound").height(260.0).show(
            ui,
            &mut st0,
            &[Layer3D::from(
                TetraMesh3D::new(&tm).name("tets").boundary_only(true),
            )],
        )
    });
    assert_eq!(r_bound.diagnostics.plot3d.output_triangles, 6);
    assert_eq!(r_bound.diagnostics.plot3d.output_segments, 9);

    // Pick centroid of face `[1, 2, 4]` on outer boundary of tet[1] while `tet_bound` is active.
    let tri_c = [2.0 / 3.0, 2.0 / 3.0, 1.0 / 3.0];
    let pos = r_bound.transform.project_data(tri_c).unwrap().0;
    let hover = run_plot3d(&ctx, vec![Event::PointerMoved(pos)], false, |ui| {
        Plot3D::new("tet_bound").height(260.0).show(
            ui,
            &mut st0,
            &[Layer3D::from(
                TetraMesh3D::new(&tm).name("tets").boundary_only(true),
            )],
        )
    });
    assert!(
        hover
            .hovered_region
            .as_ref()
            .is_some_and(|reg| reg.label.starts_with("tet["))
    );

    let mut st1 = Plot3DState::default();
    let r_all = run_plot3d(&ctx, vec![], false, |ui| {
        Plot3D::new("tet_all").height(260.0).show(
            ui,
            &mut st1,
            &[Layer3D::from(
                TetraMesh3D::new(&tm).name("tets").boundary_only(false),
            )],
        )
    });
    assert_eq!(r_all.diagnostics.plot3d.output_triangles, 8);
}
