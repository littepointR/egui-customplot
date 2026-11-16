use egui::{Color32, Context, Event};
use egui_customplot::{
    Axis3D, ConstantPlane3D, Fill3D, Layer3D, Plot3D, Plot3DError, Plot3DResponse, Plot3DState,
    Polygon3DData, Quiver3D, Quiver3DData,
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
            ui.ctx().request_discard("test plot3d vectors planes");
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
fn quiver3_renders_shafts_and_pyramid_heads_and_picks_endpoints() {
    assert_eq!(
        Quiver3DData::from_components(&[0.0], &[0.0, 1.0], &[0.0], &[1.0], &[1.0], &[1.0])
            .unwrap_err(),
        Plot3DError::LengthMismatch
    );
    let q_raw = Quiver3DData::from_components(
        &[0.0, 1.0, 0.0],
        &[0.0, 0.0, 1.0],
        &[0.0, 0.5, 0.5],
        &[0.6, -0.4, 0.3],
        &[0.4, 0.6, -0.5],
        &[0.5, 0.3, 0.8],
    )
    .unwrap();
    assert!((q_raw.scale_factor() - 1.0).abs() < 1e-12);
    let q_auto = q_raw.clone().autoscale(0.9);
    assert!(q_auto.scale_factor() > 0.0 && q_auto.scale_factor() != 1.0);

    let ctx = Context::default();
    let mut state = Plot3DState::default();
    let out = run_plot3d(&ctx, vec![], true, |ui| {
        Plot3D::new("quiver3_test").height(280.0).show(
            ui,
            &mut state,
            &[Layer3D::from(Quiver3D::new(&q_auto).name("quiver3"))],
        )
    });
    let n = q_auto.arrows().len();
    assert_eq!(out.diagnostics.plot3d.output_segments, n);
    assert!(out.diagnostics.plot3d.output_triangles >= n * 4);

    let end0 = q_auto.arrows()[0].endpoint(q_auto.scale_factor()).unwrap();
    let (end_pos, _) = out.transform.project_data(end0).unwrap();
    let hovered = run_plot3d(&ctx, vec![Event::PointerMoved(end_pos)], false, |ui| {
        Plot3D::new("quiver3_test").height(280.0).show(
            ui,
            &mut state,
            &[Layer3D::from(Quiver3D::new(&q_auto).name("quiver3"))],
        )
    });
    assert_eq!(hovered.hovered_point.unwrap().point_index, 0);
}

#[test]
fn fill3_validates_triangulates_and_picks_3d_polygons() {
    assert_eq!(
        Polygon3DData::from_polygons(&[&[[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]]]).unwrap_err(),
        Plot3DError::DegeneratePolygon
    );
    assert_eq!(
        Polygon3DData::from_polygons(&[&[[0.0, 0.0, 0.0], [1.0, 1.0, 1.0], [2.0, 2.0, 2.0]]])
            .unwrap_err(),
        Plot3DError::DegeneratePolygon
    );

    // Concave planar 3D polygon lying in z = 0.25x + 0.5.
    let z = |x: f64| 0.25 * x + 0.5;
    let concave = [
        [-0.8, -0.8, z(-0.8)],
        [0.8, -0.8, z(0.8)],
        [0.8, 0.8, z(0.8)],
        [0.0, 0.0, z(0.0)],
        [-0.8, 0.8, z(-0.8)],
    ];
    let data = Polygon3DData::from_polygons(&[&concave]).unwrap();
    assert_eq!(data.polygons()[0].triangles.len(), 3);

    let ctx = Context::default();
    let mut state = Plot3DState::default();
    let out = run_plot3d(&ctx, vec![], true, |ui| {
        Plot3D::new("fill3_test").height(280.0).show(
            ui,
            &mut state,
            &[Layer3D::from(
                Fill3D::new(&data)
                    .face_colors(&[Color32::from_rgb(217, 83, 25)])
                    .name("fill3"),
            )],
        )
    });
    assert!(out.diagnostics.plot3d.output_triangles >= 3);

    // Point [-0.4, 0.0, z(-0.4)] lies inside the concave wing.
    let (pt, _) = out.transform.project_data([-0.4, 0.0, z(-0.4)]).unwrap();
    let hovered = run_plot3d(&ctx, vec![Event::PointerMoved(pt)], false, |ui| {
        Plot3D::new("fill3_test").height(280.0).show(
            ui,
            &mut state,
            &[Layer3D::from(Fill3D::new(&data).name("fill3"))],
        )
    });
    let reg = hovered.hovered_region.expect("concave 3D polygon picked");
    assert_eq!(reg.item_index, 0);
    assert_eq!(reg.label, "fill3[0]");
}

#[test]
fn constantplane_clips_axial_and_diagonal_planes_against_axis_box() {
    let ctx = Context::default();
    let mut state = Plot3DState::default();
    let out = run_plot3d(&ctx, vec![], true, |ui| {
        Plot3D::new("constantplane_test")
            .height(280.0)
            .x_axis(Axis3D::new().bounds(-1.0, 1.0))
            .y_axis(Axis3D::new().bounds(-1.0, 1.0))
            .z_axis(Axis3D::new().bounds(-1.0, 1.0))
            .constant_planes([
                ConstantPlane3D::x_plane(0.0).label("x=0"),
                ConstantPlane3D::y_plane(0.25).label("y=0.25"),
                ConstantPlane3D::z_plane(-0.25).label("z=-0.25"),
                // Diagonal x + y + z = 0 slices [-1, 1]^3 into a hexagon (4 triangles).
                ConstantPlane3D::new([1.0, 1.0, 1.0], 0.0).label("x+y+z=0"),
            ])
            .show(ui, &mut state, &[])
    });
    assert_eq!(out.diagnostics.plot3d.output_planes, 4);
    assert!(out.diagnostics.plot3d.output_triangles >= 10);
}
