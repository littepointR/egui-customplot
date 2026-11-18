use egui_customplot::{Axis3D, AxisScale, Bounds3D, Line3D, Plot3D, Plot3DState, Point3DData};

#[test]
fn compact_titled_viewport_survives_layout_retries() {
    for height in [32., 36., 37., 38.] {
        let mut passes = 0;
        egui::Context::default()
            .run_ui(Default::default(), |ui| {
                if passes == 0 {
                    ui.ctx().request_discard("compact layout retry");
                }
                passes += 1;
                ui.set_max_width(32.);
                let response = Plot3D::new("compact").height(height).title("T").show(
                    ui,
                    &mut Plot3DState::default(),
                    &[],
                );
                assert!(response.outer_rect.contains_rect(response.plot_area));
                assert!(response.plot_area.height() > 1.);
            })
            .drop_without_applying_deltas();
        assert_eq!(passes, 2);
    }
}

#[test]
fn probability_axis_preserves_valid_samples_and_other_axes() {
    for scale in [AxisScale::Probit, AxisScale::Weibull] {
        let data =
            Point3DData::from_xyz(&[0.01, 0.99, 1.1], &[1., 2., 50.], &[5., 6., 99.]).unwrap();
        egui::Context::default()
            .run_ui(Default::default(), |ui| {
                let plot = || Plot3D::new("prob").x_axis(Axis3D::new().scale(scale));
                let mut state = Plot3DState::default();
                let layers = [Line3D::new(&data).into()];
                let res = plot().show(ui, &mut state, &layers);
                assert_eq!(res.transform.scales()[0], scale);
                assert_eq!(res.diagnostics.plot3d.domain_invalid_items, 1);
                let b = res.transform.bounds();
                assert!(b.contains([0.01, 1., 5.]) && b.contains([0.99, 2., 6.]));
                assert!(b.max()[1] < 3.);
                state.set_bounds(Bounds3D::new([-1., 10., 20.], [2., 11., 21.]).unwrap());
                let res = plot().show(ui, &mut state, &layers);
                assert_eq!(res.transform.scales()[0], scale);
                assert_eq!(res.transform.bounds().min(), [b.min()[0], 10., 20.]);
            })
            .drop_without_applying_deltas();
    }
}
#[cfg(feature = "export")]
#[derive(Debug)]
struct OrangeZ(std::sync::Mutex<Vec<egui::Pos2>>);

#[cfg(feature = "export")]
impl egui_customplot::AxisDecoration3D for OrangeZ {
    fn select_edge(
        &self,
        ctx: &egui_customplot::AxisLayoutContext3D,
        _: egui_customplot::AxisEdge3D,
    ) -> egui_customplot::AxisEdge3D {
        assert_eq!(ctx.axis, 2);
        egui_customplot::AxisEdge3D::new(true, false)
    }
    fn layout_labels(
        &self,
        _: &egui_customplot::AxisLayoutContext3D,
        edge: egui_customplot::AxisEdge3D,
        labels: &mut Vec<egui_customplot::AxisLabel3D>,
    ) {
        assert_eq!(edge, egui_customplot::AxisEdge3D::new(true, false));
        for label in labels {
            label.color = egui::Color32::ORANGE;
        }
    }
    fn paint_label(&self, painter: &egui::Painter, label: &egui_customplot::AxisLabel3D) {
        assert_eq!(label.color, egui::Color32::ORANGE);
        if label.value.is_none() {
            self.0.lock().unwrap().push(label.position);
        }
        painter.text(
            label.position,
            label.align,
            &label.text,
            label.font.clone(),
            label.color,
        );
    }
}

#[cfg(feature = "export")]
#[test]
fn per_axis_layout_replays_at_both_export_densities() {
    use egui_customplot::{PlotSnapshot, Projection3D};
    use std::sync::{Arc, Mutex};
    for proj in [
        Projection3D::Orthographic,
        Projection3D::perspective_default(),
    ] {
        let z = Arc::new(OrangeZ(Mutex::new(Vec::new())));
        let canvas = egui::vec2(360., 240.);
        let mut titles = Vec::new();
        for density in [1., 3.] {
            PlotSnapshot::render_at(
                &egui::Context::default(),
                canvas,
                density,
                Default::default(),
                |ui| {
                    Plot3D::new("axes")
                        .height(canvas.y)
                        .projection(proj)
                        .x_axis(Axis3D::new().reversed(true))
                        .z_axis(Axis3D::new().label("height").decoration(z.clone()))
                        .show(ui, &mut Default::default(), &[])
                        .outer_rect
                },
            )
            .unwrap();
            titles.push(*z.0.lock().unwrap().last().unwrap());
        }
        assert!(titles[0].distance(titles[1]) < 1.0);
    }
}

#[test]
fn oblique_lens_and_panned_camera_share_forward_and_inverse_geometry() {
    use egui_customplot::{Camera3D, CameraLens3D, Projection3D};
    let lens = CameraLens3D::new([[1., 0.2, 0.15, -0.6], [0., 1., 0., 0.]], [0., 1.]).unwrap();
    for model in [None, Some(lens)] {
        for proj in [
            Projection3D::Orthographic,
            Projection3D::perspective_default(),
        ] {
            egui::Context::default()
                .run_ui(Default::default(), |ui| {
                    let mut camera = Camera3D::view_3d().projection(proj);
                    camera.pan = [0.12, -0.18];
                    let plot = Plot3D::new("lens").camera(camera);
                    let plot = if let Some(l) = model {
                        plot.lens(l)
                    } else {
                        plot
                    };
                    let t = plot.show(ui, &mut Default::default(), &[]).transform;
                    let point = [0.12, 0.31, 0.62];
                    let (pos, depth) = t.project_unit(point).unwrap();
                    let (origin, direction, near, far) = t.screen_ray_unit(pos).unwrap();
                    let d = f64::from(depth);
                    assert!(near <= d && d <= far);
                    for i in 0..3 {
                        assert!((origin[i] + d * direction[i] - point[i]).abs() < 0.0001);
                    }
                })
                .drop_without_applying_deltas();
        }
    }
}

#[test]
fn positive_square_root_mapping_preserves_raw_units_and_rejects_invalid_samples() {
    let mapping = egui_customplot::ScaleMapping3D::new(
        AxisScale::Log10,
        |x| Some(x.sqrt()),
        |x| (x > 0.).then_some(x * x),
    )
    .unwrap();
    let data =
        Point3DData::from_xyz(&[-1., 1., 4., 9.], &[0., 1., 2., 3.], &[2., 3., 4., 5.]).unwrap();
    egui::Context::default()
        .run_ui(Default::default(), |ui| {
            let out = Plot3D::new("mapped")
                .x_axis(
                    Axis3D::new()
                        .coordinate_mapping(mapping)
                        .bounds(1., 9.)
                        .reversed(true),
                )
                .show(ui, &mut Default::default(), &[Line3D::new(&data).into()]);
            assert_eq!(out.diagnostics.plot3d.domain_invalid_items, 1);
            let t = out.transform;
            let u = t.data_to_unit([4., 2., 4.]).unwrap();
            assert!((u[0] - 0.5).abs() < 1e-10);
            assert!((t.unit_to_data(u).unwrap()[0] - 4.).abs() < 1e-10);
            assert!(t.project_data([-1., 1., 1.]).is_none());
        })
        .drop_without_applying_deltas();
}

#[test]
fn comet_picking_stops_at_the_current_head() {
    use egui_customplot::{Comet3D, Comet3DPlayback};
    let ctx = egui::Context::default();
    let data = Point3DData::from_xyz(&[0., 1., 2.], &[0.; 3], &[0.; 3]).unwrap();
    let mut playback = Comet3DPlayback::new(1.);
    playback.set_progress_points(1., 3);
    let mut state = Plot3DState::default();
    let mut pointer = None;
    for frame in 0..3 {
        let input = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(460., 320.),
            )),
            events: pointer.into_iter().map(egui::Event::PointerMoved).collect(),
            ..Default::default()
        };
        ctx.run_ui(input, |ui| {
            let out =
                Plot3D::new("comet").show(ui, &mut state, &[Comet3D::new(&data, &playback).into()]);
            if frame == 1 {
                assert!(out.hovered_point.is_none());
            }
            if frame == 2 {
                assert!(out.hovered_point.is_some());
            }
            let x = if frame == 0 { 2. } else { 1. };
            pointer = Some(out.transform.project_data([x, 0., 0.]).unwrap().0);
        })
        .drop_without_applying_deltas();
    }
}

#[test]
fn hidden_axis_grid_defaults_and_color_bar_use_resolved_style() {
    use egui_customplot::{ColorScale, PlotDefaults, PlotStyle, set_plot_defaults};
    let ctx = egui::Context::default();
    let color = egui::Color32::from_rgb(185, 42, 177);
    let stroke = egui::Stroke::new(2.25, color);
    for enabled in [true, false] {
        set_plot_defaults(
            &ctx,
            Some(PlotDefaults {
                show_grid: Some(enabled),
                ..Default::default()
            }),
        );
        let output = ctx.run_ui(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(500., 400.),
                )),
                ..Default::default()
            },
            |ui| {
                Plot3D::new("style")
                    .z_axis(Axis3D::new().label("hidden Z").visible(false))
                    .color_bar(ColorScale::new([0., 1.]).unwrap(), "Amplitude")
                    .style(PlotStyle {
                        text_color: Some(color),
                        grid_stroke: Some(stroke),
                        ..Default::default()
                    })
                    .show(ui, &mut Default::default(), &[]);
            },
        );
        let mut grid = false;
        let mut saw_label = false;
        for item in &output.shapes {
            match &item.shape {
                egui::Shape::LineSegment { stroke: s, .. } if *s == stroke => grid = true,
                egui::Shape::Text(t) => {
                    assert_ne!(t.galley.text(), "hidden Z");
                    if t.galley.text() == "Amplitude" {
                        saw_label = true;
                        assert_eq!(t.fallback_color, color);
                    }
                }
                _ => {}
            }
        }
        assert!(saw_label);
        assert_eq!(grid, enabled);
        output.drop_without_applying_deltas();
    }
}
