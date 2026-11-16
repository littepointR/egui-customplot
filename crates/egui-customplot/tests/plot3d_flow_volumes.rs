use egui::{Context, Event};
use egui_customplot::{
    Bounds3D, ConePlot3D, ConePlot3DData, Layer3D, Plot3D, Plot3DResponse, Plot3DState, Point3D,
    Quiver3D, SlicePlane3D, Stream3DOptions, StreamParticles3D, StreamParticles3DData,
    StreamParticlesPlayback, StreamRibbon3D, StreamRibbon3DData, StreamTube3D, StreamTube3DData,
    Streamline3D, Streamline3DData, VectorField3D,
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
            ui.ctx().request_discard("test stage9c flow");
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
fn streamline3d_streamslice_and_streamparticles_stay_accurate_and_idempotent() {
    let bounds = Bounds3D::new([-2.0; 3], [2.0; 3]).unwrap();
    let vf =
        VectorField3D::from_function(bounds, [16, 16, 16], 10_000, |x, y, _| [-y, x, 0.4]).unwrap();

    let sl = Streamline3DData::from_field(
        &vf,
        &[[0.8, 0.0, -1.5], [1.2, 0.0, -1.5]],
        Stream3DOptions {
            step_size: 0.05,
            max_steps: 200,
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(sl.lines().len(), 2);
    for &p in sl.lines()[0].points.iter() {
        assert!(
            (p[0].hypot(p[1]) - 0.8).abs() < 0.06,
            "helix radius drifted: {p:?}"
        );
    }

    let (slice_sl, slice_q) = Streamline3DData::streamslice(&vf, SlicePlane3D::Z(0.0), 4).unwrap();
    assert!(!slice_sl.lines().is_empty());
    assert!(!slice_q.arrows().is_empty());
    for line in slice_sl.lines() {
        for &p in line.points.iter() {
            assert!(p[2].abs() < 1e-9, "streamslice z=0 drifted to z={}", p[2]);
        }
    }

    let sp = StreamParticles3DData::from_streamlines(&sl, 5);
    let mut playback = StreamParticlesPlayback::new(0.5);
    let ctx = Context::default();
    let mut state = Plot3DState::default();

    let p_init = playback.positions(&sp);
    assert_eq!(p_init.len(), 10);

    let r1 = run_plot3d(&ctx, vec![], true, |ui| {
        playback.advance_once(ui.ctx().cumulative_frame_nr(), 0.25);
        Plot3D::new("flow_particles").height(260.0).show(
            ui,
            &mut state,
            &[
                Layer3D::from(Streamline3D::new(&slice_sl).name("slice_lines")),
                Layer3D::from(Quiver3D::new(&slice_q).name("slice_arrows")),
                Layer3D::from(StreamParticles3D::new(&sp, &playback).name("particles")),
            ],
        )
    });
    assert!((playback.phase() - 0.125).abs() < 1e-12);
    assert_eq!(r1.diagnostics.plot3d.output_markers, 10);

    let p_after = playback.positions(&sp);
    assert_ne!(p_init[0].0, p_after[0].0);

    // Pick animated particle at its live frame position (keep identical layers & bounds).
    let pos0 = r1.transform.project_data(p_after[0].0).unwrap().0;
    let hover = run_plot3d(&ctx, vec![Event::PointerMoved(pos0)], false, |ui| {
        Plot3D::new("flow_particles").height(260.0).show(
            ui,
            &mut state,
            &[
                Layer3D::from(Streamline3D::new(&slice_sl).name("slice_lines")),
                Layer3D::from(Quiver3D::new(&slice_q).name("slice_arrows")),
                Layer3D::from(StreamParticles3D::new(&sp, &playback).name("particles")),
            ],
        )
    });
    assert!(hover.hovered_point.is_some());
}

#[test]
fn streamribbon_twists_under_curl_streamtube_scales_under_divergence_and_coneplot_renders() {
    let bounds = Bounds3D::new([-1.5; 3], [1.5; 3]).unwrap();
    let opts = Stream3DOptions {
        step_size: 0.05,
        max_steps: 100,
        ..Default::default()
    };

    let rot_vf =
        VectorField3D::from_function(bounds, [14, 14, 14], 8_000, |x, y, _| [-y, x, 1.0]).unwrap();
    let irrot_vf =
        VectorField3D::from_function(bounds, [14, 14, 14], 8_000, |_, _, _| [0.0, 0.0, 1.0])
            .unwrap();

    let rot_rib =
        StreamRibbon3DData::from_field(&rot_vf, &[[0.5, 0.0, -1.0]], 0.2, 1.0, opts).unwrap();
    let irrot_rib =
        StreamRibbon3DData::from_field(&irrot_vf, &[[0.5, 0.0, -1.0]], 0.2, 1.0, opts).unwrap();
    assert!(rot_rib.total_twist_rad()[0].abs() > 0.5);
    assert!(irrot_rib.total_twist_rad()[0].abs() < 1e-9);

    let exp_vf = VectorField3D::from_function(bounds, [14, 14, 14], 8_000, |x, y, _| {
        [0.4 * x, 0.4 * y, 1.0]
    })
    .unwrap();
    let exp_tube =
        StreamTube3DData::from_field(&exp_vf, &[[0.0, 0.0, -1.0]], 0.15, 12, opts).unwrap();
    let const_tube =
        StreamTube3DData::from_field(&irrot_vf, &[[0.0, 0.0, -1.0]], 0.15, 12, opts).unwrap();

    let last_ring_radius = |tube_data: &StreamTube3DData| -> f64 {
        let surf = &tube_data.tubes()[0];
        let r = surf.rows() - 1;
        let c0 = surf.cols();
        let pts: Vec<Point3D> = (0..c0 - 1).map(|c| surf.point(r, c).unwrap()).collect();
        let center = [
            pts.iter().map(|p| p[0]).sum::<f64>() / pts.len() as f64,
            pts.iter().map(|p| p[1]).sum::<f64>() / pts.len() as f64,
            pts.iter().map(|p| p[2]).sum::<f64>() / pts.len() as f64,
        ];
        let d = [
            pts[0][0] - center[0],
            pts[0][1] - center[1],
            pts[0][2] - center[2],
        ];
        (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt()
    };
    let r_exp = last_ring_radius(&exp_tube);
    let r_const = last_ring_radius(&const_tube);
    assert!(
        r_const > 0.14 && r_const < 0.16,
        "const tube radius: {r_const}"
    );
    assert!(
        r_exp > r_const + 0.04,
        "expected divergence expansion: exp={r_exp}, const={r_const}"
    );

    let cones = ConePlot3DData::from_field(
        &rot_vf,
        &[[0.5, 0.0, 0.0], [0.0, 0.5, 0.3], [-0.5, 0.0, -0.3]],
        0.35,
        10,
    )
    .unwrap();
    assert_eq!(cones.cones().len(), 3);

    let ctx = Context::default();
    let mut state = Plot3DState::default();
    let resp = run_plot3d(&ctx, vec![], true, |ui| {
        Plot3D::new("geom_flow_test").height(260.0).show(
            ui,
            &mut state,
            &[
                Layer3D::from(StreamRibbon3D::new(&rot_rib).name("ribbon")),
                Layer3D::from(StreamTube3D::new(&exp_tube).name("tube")),
                Layer3D::from(ConePlot3D::new(&cones).name("cones")),
            ],
        )
    });
    assert!(resp.diagnostics.plot3d.output_triangles > 300);

    let pos = resp
        .transform
        .project_data(cones.cones()[0].apex)
        .unwrap()
        .0;
    let hover = run_plot3d(&ctx, vec![Event::PointerMoved(pos)], false, |ui| {
        Plot3D::new("geom_flow_test").height(260.0).show(
            ui,
            &mut state,
            &[
                Layer3D::from(StreamRibbon3D::new(&rot_rib).name("ribbon")),
                Layer3D::from(StreamTube3D::new(&exp_tube).name("tube")),
                Layer3D::from(ConePlot3D::new(&cones).name("cones")),
            ],
        )
    });
    assert!(hover.hovered_point.is_some() || hover.hovered_region.is_some());
}
