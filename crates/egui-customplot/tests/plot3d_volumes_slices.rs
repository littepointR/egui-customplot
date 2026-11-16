use egui::{Context, Event};
use egui_customplot::{
    Bounds3D, ColorScale, Contour3D, Contour3DData, ContourLevels, Layer3D, Plot3D, Plot3DResponse,
    Plot3DState, ScalarVolume3D, SlicePlane3D, VolumeSlice3D, VolumeSliceData,
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
            ui.ctx().request_discard("test stage9c volumes slices");
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
fn scalar_volume_trilinear_sampling_gradients_bounds_and_nan_safety() {
    let bounds = Bounds3D::new([-1.0, -2.0, -0.5], [2.0, 1.0, 2.5]).unwrap();
    let vol = ScalarVolume3D::from_function(bounds, [6, 6, 6], 1_000, |x, y, z| {
        2.0 * x - 3.0 * y + 4.0 * z
    })
    .unwrap();
    assert_eq!(vol.dims(), [6, 6, 6]);
    assert_eq!(vol.non_finite(), 0);

    let p = [0.37, -0.41, 1.13];
    let expected = 2.0 * p[0] - 3.0 * p[1] + 4.0 * p[2];
    let val = vol.sample(p).unwrap();
    assert!(
        (val - expected).abs() < 1e-11,
        "got {val}, expected {expected}"
    );

    let grad = vol.gradient(p).unwrap();
    assert!((grad[0] - 2.0).abs() < 1e-10);
    assert!((grad[1] + 3.0).abs() < 1e-10);
    assert!((grad[2] - 4.0).abs() < 1e-10);

    assert!(vol.sample([2.5, 0.0, 0.0]).is_none());

    let mut vals = vol.values().to_vec();
    vals[0] = f64::NAN;
    let nan_vol =
        ScalarVolume3D::new(vol.x().to_vec(), vol.y().to_vec(), vol.z().to_vec(), vals).unwrap();
    assert_eq!(nan_vol.non_finite(), 1);
    assert!(nan_vol.sample([-0.9, -1.9, -0.4]).is_none());
}

#[test]
fn slice_orthogonal_and_arbitrary_planes_render_and_pick_regions() {
    let bounds = Bounds3D::new([-1.5; 3], [1.5; 3]).unwrap();
    let vol =
        ScalarVolume3D::from_function(bounds, [12, 12, 12], 5_000, |x, y, z| x * x + y * y + z * z)
            .unwrap();

    let slices = VolumeSliceData::from_volume(
        &vol,
        &[
            SlicePlane3D::X(0.0),
            SlicePlane3D::Y(0.25),
            SlicePlane3D::Z(-0.25),
            SlicePlane3D::Arbitrary {
                normal: [1.0, 1.0, 1.0],
                offset: 0.3,
                resolution: 16,
            },
        ],
    )
    .unwrap();
    assert_eq!(slices.slices().len(), 4);

    let cs = ColorScale::new([0.0, 6.75]).unwrap();
    let ctx = Context::default();
    let mut state = Plot3DState::default();
    let resp = run_plot3d(&ctx, vec![], true, |ui| {
        Plot3D::new("slice_test")
            .height(260.0)
            .color_bar(cs, "V")
            .show(
                ui,
                &mut state,
                &[Layer3D::from(
                    VolumeSlice3D::new(&slices)
                        .name("vol_slices")
                        .color_scale(cs),
                )],
            )
    });
    assert!(resp.diagnostics.plot3d.output_triangles > 200);

    // Move pointer over center of Z = -0.25 slice.
    let center_pos = resp.transform.project_data([0.0, 0.0, -0.25]).unwrap().0;
    let hover = run_plot3d(&ctx, vec![Event::PointerMoved(center_pos)], false, |ui| {
        Plot3D::new("slice_test").height(260.0).show(
            ui,
            &mut state,
            &[Layer3D::from(
                VolumeSlice3D::new(&slices).name("vol_slices"),
            )],
        )
    });
    assert!(
        hover
            .hovered_region
            .as_ref()
            .is_some_and(|reg| reg.label.starts_with("slice["))
    );
}

#[test]
fn contourslice_extracts_isolines_on_orthogonal_and_tilted_volume_planes() {
    let bounds = Bounds3D::new([-1.5; 3], [1.5; 3]).unwrap();
    let vol = ScalarVolume3D::from_function(bounds, [18, 18, 18], 10_000, |x, y, z| {
        x * x + y * y + z * z
    })
    .unwrap();

    let cs = Contour3DData::from_volume_slices(
        &vol,
        &[
            SlicePlane3D::X(0.5),
            SlicePlane3D::Arbitrary {
                normal: [0.0, 1.0, 1.0],
                offset: 0.2,
                resolution: 20,
            },
        ],
        ContourLevels::Explicit(vec![0.8, 1.4]),
    )
    .unwrap();
    assert!(!cs.lines().is_empty());

    for line in cs.lines() {
        for &p in line.points.iter() {
            let v_true = p[0] * p[0] + p[1] * p[1] + p[2] * p[2];
            assert!(
                (v_true - line.level).abs() < 0.08,
                "contourslice pt {p:?} off level {} (V={v_true})",
                line.level
            );
            let on_x = (p[0] - 0.5).abs() < 1e-6;
            let on_tilt = ((p[1] + p[2]) - 0.2).abs() < 1e-6;
            assert!(
                on_x || on_tilt,
                "pt {p:?} does not lie on either slice plane"
            );
        }
    }

    let ctx = Context::default();
    let mut state = Plot3DState::default();
    let resp = run_plot3d(&ctx, vec![], true, |ui| {
        Plot3D::new("contourslice_test").height(260.0).show(
            ui,
            &mut state,
            &[Layer3D::from(Contour3D::new(&cs).name("contourslice"))],
        )
    });
    assert!(resp.diagnostics.plot3d.output_segments >= 12);
}
