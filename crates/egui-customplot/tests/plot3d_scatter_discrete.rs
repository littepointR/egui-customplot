use egui::{Color32, Context, Event};
use egui_customplot::{
    Axis3D, Bar3D, Bar3DData, Bar3DMode, Bar3DOrientation, Bubble3D, Bubble3DData, BubbleLegend,
    Camera3D, ColorMap, ColorScale, Layer3D, Pie3D, Pie3DData, Plot3D, Plot3DResponse, Plot3DState,
    Point3DData, Scatter3D, Stem3D, Stem3DData, Swarm3D, Swarm3DData,
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
            ui.ctx().request_discard("test plot3d scatter discrete");
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
fn scatter3_picks_foreground_point_along_same_viewing_ray() {
    // Under Camera3D::view_2d_xy() (view(0, 90)), +Z is nearer the viewer (smaller depth).
    let pts = Point3DData::from_xyz(&[0.0, 0.0], &[0.0, 0.0], &[-1.0, 1.0]).unwrap();
    let ctx = Context::default();
    let mut state = Plot3DState::default();
    let first = run_plot3d(&ctx, vec![], false, |ui| {
        Plot3D::new("scatter3_depth")
            .height(280.0)
            .camera(Camera3D::view_2d_xy())
            .x_axis(Axis3D::new().bounds(-2.0, 2.0))
            .y_axis(Axis3D::new().bounds(-2.0, 2.0))
            .z_axis(Axis3D::new().bounds(-2.0, 2.0))
            .show(
                ui,
                &mut state,
                &[Layer3D::from(Scatter3D::new(&pts).name("pts"))],
            )
    });
    let (near_pos, _) = first.transform.project_data([0.0, 0.0, 1.0]).unwrap();
    let hovered = run_plot3d(&ctx, vec![Event::PointerMoved(near_pos)], true, |ui| {
        Plot3D::new("scatter3_depth")
            .height(280.0)
            .camera(Camera3D::view_2d_xy())
            .x_axis(Axis3D::new().bounds(-2.0, 2.0))
            .y_axis(Axis3D::new().bounds(-2.0, 2.0))
            .z_axis(Axis3D::new().bounds(-2.0, 2.0))
            .show(
                ui,
                &mut state,
                &[Layer3D::from(Scatter3D::new(&pts).name("pts"))],
            )
    });
    let hit = hovered.hovered_point.expect("near 3D scatter point picked");
    assert_eq!(hit.point_index, 1);
    assert_eq!(hit.position, [0.0, 0.0, 1.0]);
}

#[test]
fn bubblechart3_and_swarmchart3_render_and_preserve_raw_indices() {
    let bubble = Bubble3DData::new(
        &[[0.2, 0.3, 1.0], [1.4, 0.9, 2.2], [2.1, 1.7, 3.5]],
        &[12.0, 48.0, 96.0],
    )
    .unwrap()
    .with_scalars(&[0.1, 0.5, 0.9])
    .unwrap()
    .size_range([5.0, 20.0]);
    let cs = ColorScale::new([0.0, 1.0])
        .unwrap()
        .color_map(ColorMap::Viridis);

    let g0 = [0.5, 0.52, 0.55, 0.58, 1.6, 1.62];
    let g1 = [1.0, 1.03, 1.05, 1.08];
    let swarm = Swarm3DData::from_groups(&[[0.0, 0.0], [2.0, 1.0]], &[&g0, &g1], 0.4).unwrap();
    for pt in swarm.points() {
        let dx = pt.position[0] - pt.raw_xy[0];
        let dy = pt.position[1] - pt.raw_xy[1];
        assert!((dx * dx + dy * dy).sqrt() <= 0.4 + 1e-9);
    }

    let ctx = Context::default();
    let mut b_state = Plot3DState::default();
    let b_resp = run_plot3d(&ctx, vec![], true, |ui| {
        Plot3D::new("bubble3_plot")
            .height(280.0)
            .bubble_legend(
                BubbleLegend::new(bubble.size_extent(), bubble.radius_range())
                    .steps(3)
                    .title("Size"),
            )
            .show(
                ui,
                &mut b_state,
                &[Layer3D::from(
                    Bubble3D::new(&bubble).color_scale(cs).name("bubble3"),
                )],
            )
    });
    assert_eq!(
        b_resp.diagnostics.plot3d.output_markers,
        bubble.points().len()
    );

    let mut s_state = Plot3DState::default();
    let s_first = run_plot3d(&ctx, vec![], false, |ui| {
        Plot3D::new("swarm3_plot").height(280.0).show(
            ui,
            &mut s_state,
            &[Layer3D::from(
                Swarm3D::new(&swarm)
                    .group_colors(&[
                        Color32::from_rgb(0, 114, 189),
                        Color32::from_rgb(217, 83, 25),
                    ])
                    .name("swarm3"),
            )],
        )
    });
    assert_eq!(s_first.diagnostics.plot3d.output_markers, swarm.len());

    // Hovering a jittered sample returns its original `raw_index` within the group.
    let target = swarm
        .points()
        .iter()
        .find(|p| p.group_index == 1 && p.raw_index == 2)
        .unwrap();
    let (screen_pos, _) = s_first.transform.project_data(target.position).unwrap();
    let s_hover = run_plot3d(&ctx, vec![Event::PointerMoved(screen_pos)], false, |ui| {
        Plot3D::new("swarm3_plot").height(280.0).show(
            ui,
            &mut s_state,
            &[Layer3D::from(Swarm3D::new(&swarm).name("swarm3"))],
        )
    });
    if let Some(hit) = s_hover.hovered_point {
        assert_eq!(hit.position, target.position);
        assert_eq!(hit.point_index, target.raw_index);
    }
}

#[test]
fn bar3_bar3h_and_stem3_render_all_modes_and_support_hover() {
    let values = [1.2, 2.4, 0.8, 1.8, 1.5, 2.1];
    let ctx = Context::default();

    for mode in [Bar3DMode::Detached, Bar3DMode::Grouped, Bar3DMode::Stacked] {
        for orientation in [Bar3DOrientation::Vertical, Bar3DOrientation::Horizontal] {
            let bars = Bar3DData::from_matrix(&values, 3, 2, mode, orientation, 0.75).unwrap();
            let mut state = Plot3DState::default();
            let first = run_plot3d(&ctx, vec![], true, |ui| {
                Plot3D::new(("bar3", mode as u8, orientation as u8))
                    .height(280.0)
                    .show(
                        ui,
                        &mut state,
                        &[Layer3D::from(Bar3D::new(&bars).name("bars"))],
                    )
            });
            assert!(
                first.diagnostics.plot3d.output_triangles >= bars.boxes().len() * 12,
                "each bar prism emits 12 triangles across 6 faces"
            );

            // Pick center of first bar box.
            let b0 = &bars.boxes()[0];
            let center = [
                0.5 * (b0.min[0] + b0.max[0]),
                0.5 * (b0.min[1] + b0.max[1]),
                0.5 * (b0.min[2] + b0.max[2]),
            ];
            let (pos, _) = first.transform.project_data(center).unwrap();
            let hovered = run_plot3d(&ctx, vec![Event::PointerMoved(pos)], false, |ui| {
                Plot3D::new(("bar3", mode as u8, orientation as u8))
                    .height(280.0)
                    .show(
                        ui,
                        &mut state,
                        &[Layer3D::from(Bar3D::new(&bars).name("bars"))],
                    )
            });
            let bar_hit = hovered.hovered_bar.expect("hovered bar3 prism");
            assert!(bar_hit.value.is_finite());
        }
    }

    let stems = Stem3DData::new(
        vec![
            [0.0, 0.0, 1.5],
            [1.0, 0.5, 2.2],
            [0.4, 1.2, 0.9],
            [1.4, 1.3, 1.8],
        ],
        0.2,
    )
    .unwrap();
    let mut s_state = Plot3DState::default();
    let s_out = run_plot3d(&ctx, vec![], true, |ui| {
        Plot3D::new("stem3_test").height(260.0).show(
            ui,
            &mut s_state,
            &[Layer3D::from(Stem3D::new(&stems).name("stem3"))],
        )
    });
    assert!(s_out.diagnostics.plot3d.output_segments >= stems.points().len());
    assert!(s_out.diagnostics.plot3d.output_markers >= stems.points().len());
}

#[test]
fn pie3_renders_extruded_cylinder_wedges_and_picks_hovered_slice() {
    let pie = Pie3DData::with_options(
        &[30.0, 45.0, 25.0],
        &[false, true, false],
        &["Alpha", "Beta", "Gamma"],
        0.35,
    )
    .unwrap();

    let ctx = Context::default();
    let mut state = Plot3DState::default();
    let first = run_plot3d(&ctx, vec![], true, |ui| {
        Plot3D::new("pie3_test")
            .height(280.0)
            .camera(Camera3D::view_3d())
            .show(
                ui,
                &mut state,
                &[Layer3D::from(Pie3D::new(&pie).name("pie3"))],
            )
    });
    assert!(first.diagnostics.plot3d.output_triangles > 40);

    // Pick near the top wedge centroid of slice 1 ("Beta", exploded).
    let s1 = &pie.slices()[1];
    let mid = 0.5 * (s1.start_angle_rad + s1.end_angle_rad);
    let r = pie.offset() + 0.55;
    let pick_pt = [r * mid.cos(), r * mid.sin(), pie.thickness()];
    let (screen_pos, _) = first.transform.project_data(pick_pt).unwrap();
    let hovered = run_plot3d(&ctx, vec![Event::PointerMoved(screen_pos)], false, |ui| {
        Plot3D::new("pie3_test")
            .height(280.0)
            .camera(Camera3D::view_3d())
            .show(
                ui,
                &mut state,
                &[Layer3D::from(Pie3D::new(&pie).name("pie3"))],
            )
    });
    let reg = hovered.hovered_region.expect("pie3 top wedge picked");
    assert_eq!(reg.item_index, 1);
    assert_eq!(reg.label, "Beta");
}
