use std::f64::consts::TAU;

use egui::{Context, Event};
use egui_customplot::{
    Bounds3D, FunctionContourError, ImplicitSurface3DData, Layer3D, Plot3D, Plot3DResponse,
    Plot3DState, Point3D, Surface3D, Surface3DData, TriMesh3DData, TriMeshError, TriSurf3D,
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
            ui.ctx().request_discard("test stage9b analytic implicit");
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
fn fsurf_parametric_and_sphere_cylinder_ellipsoid_geometries_are_exact() {
    assert!(matches!(
        Surface3DData::from_parametric_fn([0.0, TAU], [0.0, TAU], [12, 12], 50, |u, v| [
            u.cos(),
            u.sin(),
            v
        ]),
        Err(FunctionContourError::BudgetExceeded { .. })
    ));

    let torus =
        Surface3DData::from_parametric_fn([0.0, TAU], [0.0, TAU], [20, 16], 10_000, |u, v| {
            let r = 2.0 + 0.65 * v.cos();
            [r * u.cos(), r * u.sin(), 0.65 * v.sin()]
        })
        .unwrap();
    let ez_torus = Surface3DData::ez_parametric(1_600, |u, v| {
        let r = 2.0 + 0.5 * v.cos();
        [r * u.cos(), r * u.sin(), 0.5 * v.sin()]
    })
    .unwrap();
    assert_eq!(torus.rows(), 16);
    assert_eq!(torus.cols(), 20);
    assert_eq!(ez_torus.rows(), 40);

    let sph = Surface3DData::sphere(16).unwrap();
    for p in sph.points() {
        let r2 = p[0] * p[0] + p[1] * p[1] + p[2] * p[2];
        assert!((r2 - 1.0).abs() < 1e-12, "unit sphere error: {r2}");
    }

    let cyl = Surface3DData::cylinder(&[1.0, 0.5, 1.2], 16).unwrap();
    assert_eq!(cyl.rows(), 3);
    assert_eq!(cyl.cols(), 17);
    assert!((cyl.z_range().unwrap()[0] - 0.0).abs() < 1e-12);
    assert!((cyl.z_range().unwrap()[1] - 1.0).abs() < 1e-12);

    let center = [1.0, -1.0, 2.0];
    let radii = [2.0, 3.0, 4.0];
    let ell = Surface3DData::ellipsoid(center, radii, 12).unwrap();
    for (p, n) in ell.points().iter().zip(ell.normals().iter()) {
        let dx = (p[0] - center[0]) / radii[0];
        let dy = (p[1] - center[1]) / radii[1];
        let dz = (p[2] - center[2]) / radii[2];
        let q = dx * dx + dy * dy + dz * dz;
        assert!((q - 1.0).abs() < 1e-12);
        let outward = dx * n[0] + dy * n[1] + dz * n[2];
        assert!(outward > 0.0);
    }

    let ctx = Context::default();
    let mut st = Plot3DState::default();
    let resp = run_plot3d(&ctx, vec![], true, |ui| {
        Plot3D::new("analytic_geom").height(260.0).show(
            ui,
            &mut st,
            &[
                Layer3D::from(Surface3D::new(&torus).name("torus")),
                Layer3D::from(Surface3D::new(&cyl).name("cyl")),
                Layer3D::from(Surface3D::new(&ell).name("ell")),
            ],
        )
    });
    assert!(resp.diagnostics.plot3d.output_triangles > 400);
}

#[test]
fn fimplicit3_extracts_unit_sphere_handles_empty_and_guards_poles() {
    let bounds = Bounds3D::new([-1.5, -1.5, -1.5], [1.5, 1.5, 1.5]).unwrap();
    let imp = ImplicitSurface3DData::from_function(bounds, [24, 24, 24], 50_000, |x, y, z| {
        x * x + y * y + z * z - 1.0
    })
    .unwrap();
    let mesh = imp.mesh().expect("non-empty unit sphere zero level set");
    assert_eq!(imp.evaluations(), 24 * 24 * 24);
    assert!(!mesh.triangles().is_empty());

    for (v, n) in mesh.vertices().iter().zip(mesh.vertex_normals().iter()) {
        let r2 = v[0] * v[0] + v[1] * v[1] + v[2] * v[2];
        assert!((r2 - 1.0).abs() < 0.08, "vertex off unit sphere: r^2={r2}");
        let dot = v[0] * n[0] + v[1] * n[1] + v[2] * n[2];
        assert!(
            dot > 0.90,
            "expected outward gradient normal, got dot={dot}"
        );
    }

    let empty = ImplicitSurface3DData::from_function(bounds, [12, 12, 12], 5_000, |x, y, z| {
        x * x + y * y + z * z + 5.0
    })
    .unwrap();
    assert!(empty.mesh().is_none());

    // Pole discontinuity guard on 1 / (x + y + z - 0.123).
    let pole = ImplicitSurface3DData::from_function(bounds, [18, 18, 18], 20_000, |x, y, z| {
        1.0 / (x + y + z - 0.123)
    })
    .unwrap();
    assert!(pole.discontinuities() > 0);
    assert!(pole.mesh().is_none());

    let ctx = Context::default();
    let mut st0 = Plot3DState::default();
    let resp0 = run_plot3d(&ctx, vec![], false, |ui| {
        Plot3D::new("empty_imp").height(240.0).show(
            ui,
            &mut st0,
            &[Layer3D::from(
                TriSurf3D::from_implicit(&empty).name("empty"),
            )],
        )
    });
    assert_eq!(resp0.diagnostics.plot3d.output_triangles, 0);

    let mut st1 = Plot3DState::default();
    let resp1 = run_plot3d(&ctx, vec![], true, |ui| {
        Plot3D::new("sphere_imp").height(240.0).show(
            ui,
            &mut st1,
            &[Layer3D::from(TriSurf3D::from_implicit(&imp).name("sphere"))],
        )
    });
    assert_eq!(
        resp1.diagnostics.plot3d.output_triangles,
        mesh.triangles().len()
    );
}

#[test]
fn trisurf_and_3d_trimesh_validate_pick_and_respect_hidden_line() {
    assert_eq!(
        TriMesh3DData::new(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 9]],
        )
        .unwrap_err(),
        TriMeshError::InvalidTriangleIndex
    );

    let mut pts: Vec<Point3D> = Vec::new();
    for j in 0..5 {
        for i in 0..5 {
            let x = i as f64 - 2.0;
            let y = j as f64 - 2.0;
            let z = 1.5 - 0.25 * (x * x + y * y);
            pts.push([x, y, z]);
        }
    }
    let mesh = TriMesh3DData::from_delaunay_xy(&pts).unwrap();
    assert!(!mesh.edges().is_empty());

    let ctx = Context::default();
    let mut st = Plot3DState::default();
    let shaded = run_plot3d(&ctx, vec![], true, |ui| {
        Plot3D::new("trisurf_test").height(260.0).show(
            ui,
            &mut st,
            &[Layer3D::from(TriSurf3D::new(&mesh).name("ts"))],
        )
    });
    assert_eq!(
        shaded.diagnostics.plot3d.output_triangles,
        mesh.triangles().len()
    );

    let [i0, i1, i2] = mesh.triangles()[0];
    let vs = mesh.vertices();
    let c = [
        (vs[i0 as usize][0] + vs[i1 as usize][0] + vs[i2 as usize][0]) / 3.0,
        (vs[i0 as usize][1] + vs[i1 as usize][1] + vs[i2 as usize][1]) / 3.0,
        (vs[i0 as usize][2] + vs[i1 as usize][2] + vs[i2 as usize][2]) / 3.0,
    ];
    let pos = shaded.transform.project_data(c).unwrap().0;
    let picked = run_plot3d(&ctx, vec![Event::PointerMoved(pos)], false, |ui| {
        Plot3D::new("trisurf_test").height(260.0).show(
            ui,
            &mut st,
            &[Layer3D::from(TriSurf3D::new(&mesh).name("ts"))],
        )
    });
    assert!(
        picked
            .hovered_region
            .as_ref()
            .is_some_and(|reg| reg.label.starts_with("trisurf["))
    );

    let mut st_wf = Plot3DState::default();
    let wf_open = run_plot3d(&ctx, vec![], false, |ui| {
        Plot3D::new("trimesh_open").height(260.0).show(
            ui,
            &mut st_wf,
            &[Layer3D::from(
                TriSurf3D::wireframe(&mesh).hidden_line(false),
            )],
        )
    });
    assert_eq!(wf_open.diagnostics.plot3d.output_triangles, 0);
    assert_eq!(
        wf_open.diagnostics.plot3d.output_segments,
        mesh.edges().len()
    );

    let mut st_wfh = Plot3DState::default();
    let wf_hidden = run_plot3d(&ctx, vec![], false, |ui| {
        Plot3D::new("trimesh_hidden").height(260.0).show(
            ui,
            &mut st_wfh,
            &[Layer3D::from(TriSurf3D::wireframe(&mesh).hidden_line(true))],
        )
    });
    assert_eq!(
        wf_hidden.diagnostics.plot3d.output_triangles,
        mesh.triangles().len()
    );
    assert_eq!(
        wf_hidden.diagnostics.plot3d.output_segments,
        mesh.edges().len()
    );
}
