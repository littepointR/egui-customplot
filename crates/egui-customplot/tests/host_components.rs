use egui::{Color32, IdSalt, Pos2, Ui};
use egui_customplot::*;
use std::sync::Arc;

#[derive(Debug)]
struct Interval {
    end: f64,
}
impl CustomPlotLayer for Interval {
    fn id(&self) -> IdSalt {
        IdSalt::new("interval")
    }
    fn name(&self) -> &str {
        "calibration interval"
    }
    fn extent(&self, _: [AxisScale; 2]) -> Option<(PlotPoint, PlotPoint)> {
        Some(([10., 5.], [self.end, 5.]))
    }
    fn paint(&self, ctx: &LayerPaintContext<'_>) {
        let line = [[10., 5.], [self.end, 5.]].map(|p| ctx.transform.position_from_point(p));
        ctx.painter.line_segment(line, ctx.stroke);
    }
    fn pick(&self, _: PlotTransform, _: Pos2, _: f32) -> Option<(usize, PlotPoint)> {
        Some((1, [self.end, 5.]))
    }
}

struct ButtonLegend;
impl LegendComponent for ButtonLegend {
    fn measure(&self, _: &Ui, names: &[&str]) -> egui::Vec2 {
        assert!(names.contains(&"calibration interval"));
        egui::vec2(160., 90.)
    }
    fn item(&self, ui: &mut Ui, name: &str, color: Color32, visible: bool) -> egui::Response {
        ui.selectable_label(visible, egui::RichText::new(name).color(color))
    }
}

#[path = "support/input.rs"]
mod support;

#[test]
fn custom_layer_uses_shared_transform_and_measured_legend() {
    let ctx = egui::Context::default();
    let mut state = PlotState::default();
    let mut interval = Interval { end: 20. };
    let base = PlotData::new(vec![[0., 0.], [1., 1.]]);
    let mut pointer = None;
    for frame in 0..3 {
        if frame == 2 {
            interval.end = 30.;
        }
        let events = pointer.into_iter().map(egui::Event::PointerMoved).collect();
        ctx.run_ui(support::input(events), |ui| {
            let mut layers = [Line::new(&base).into(), PlotLayer::custom(&interval)];
            if frame == 1 {
                layers.reverse();
            }
            let p = Plot::new("layers")
                .legend_position(LegendPosition::OutsideRight)
                .legend_component(Arc::new(ButtonLegend))
                .show_layers(ui, &mut state, &layers);
            let pos = p.transform.position_from_point([interval.end, 5.]);
            assert!(p.transform.rect().contains(pos));
            assert_eq!(p.legend_entries[0].id, interval.id());
            assert!(p.legend_entries[0].rect.left() > p.transform.rect().right());
            if frame == 1 {
                let hit = p.hovered_point.unwrap();
                assert_eq!(
                    (hit.series_id, hit.point_index, hit.series_index),
                    (interval.id(), 1, 0)
                );
            }
            pointer = Some(pos);
        })
        .drop_without_applying_deltas();
    }
    let data = Point3DData::from_xyz(&[0., 1.], &[0., 1.], &[0., 1.]).unwrap();
    ctx.run_ui(support::input(vec![]), |ui| {
        let p = Plot3D::new("3d")
            .legend_component(Arc::new(ButtonLegend))
            .show(
                ui,
                &mut Default::default(),
                &[Line3D::new(&data).name(interval.name()).into()],
            );
        assert!(p.legend_entries[0].rect.left() > p.plot_area.right());
    })
    .drop_without_applying_deltas();
}

#[cfg(feature = "export")]
#[test]
fn custom_batches_can_replace_gpu_drawing_during_capture() {
    #[derive(Debug)]
    struct ExportOnly;
    impl CustomPlotLayer for ExportOnly {
        fn id(&self) -> IdSalt {
            IdSalt::new("export")
        }
        fn extent(&self, _: [AxisScale; 2]) -> Option<(PlotPoint, PlotPoint)> {
            None
        }
        fn paint(&self, ctx: &LayerPaintContext<'_>) {
            assert!(ctx.exporting && ctx.pixels_per_point == 2.);
            ctx.painter
                .circle_filled(ctx.transform.rect().center(), 8., Color32::RED);
        }
    }
    let snapshot = PlotSnapshot::render_at(
        &egui::Context::default(),
        egui::vec2(320., 200.),
        2.,
        Default::default(),
        |ui| {
            Plot::new("capture")
                .height(200.)
                .show_layers(
                    ui,
                    &mut Default::default(),
                    &[PlotLayer::custom(&ExportOnly)],
                )
                .outer_rect
        },
    )
    .unwrap();
    assert!(
        snapshot
            .to_png(Default::default())
            .unwrap()
            .starts_with(b"\x89PNG")
    );
    assert!(!snapshot.to_svg(Default::default()).unwrap().is_empty());
}

#[test]
fn custom_3d_batches_reuse_and_invalidate_depth_cache() {
    use std::sync::atomic::{AtomicUsize, Ordering};
    #[derive(Debug)]
    struct Track {
        generation: u64,
        builds: AtomicUsize,
    }
    impl CustomLayer3D for Track {
        fn id(&self) -> IdSalt {
            IdSalt::new("track")
        }
        fn name(&self) -> &str {
            "track"
        }
        fn revision(&self) -> u64 {
            self.generation
        }
        fn extent(&self, _: [AxisScale; 3]) -> Option<[Point3D; 2]> {
            Some([[0.; 3], [1.; 3]])
        }
        fn batches(&self, _: Transform3D, submit: &mut dyn FnMut(GeometryBatch3D<'_>)) {
            self.builds.fetch_add(1, Ordering::Relaxed);
            let pts = [[0.; 3], [0.5; 3], [1.; 3]];
            submit(GeometryBatch3D::Points(&pts));
            submit(GeometryBatch3D::Polyline(&pts));
        }
        fn pick(&self, _: Transform3D, _: Pos2, _: f32) -> Option<(usize, Point3D)> {
            Some((1, [0.5; 3]))
        }
    }
    let ctx = egui::Context::default();
    let mut track = Track {
        generation: 1,
        builds: AtomicUsize::new(0),
    };
    let mut state = Plot3DState::default();
    let mut pointer = None;
    for frame in 0..3 {
        if frame == 2 {
            track.generation += 1;
        }
        ctx.run_ui(
            support::input(pointer.into_iter().map(egui::Event::PointerMoved).collect()),
            |ui| {
                let r = Plot3D::new("track").depth_mode(DepthMode::ZBuffer).show(
                    ui,
                    &mut state,
                    &[Layer3D::Custom(&track)],
                );
                assert_eq!(r.diagnostics.plot3d.output_markers, 3);
                assert_eq!(r.diagnostics.plot3d.output_segments, 2);
                if frame > 0 {
                    assert!(r.hovered_point.is_some());
                }
                pointer = Some(r.transform.project_data([0.5; 3]).unwrap().0);
            },
        )
        .drop_without_applying_deltas();
        assert_eq!(
            track.builds.load(Ordering::Relaxed),
            if frame == 2 { 2 } else { 1 }
        );
    }
}

#[test]
fn geographic_and_polar_components_reserve_space_before_navigation() {
    use std::sync::Mutex;
    struct Heading(Mutex<Option<egui::Rect>>);
    impl PlotDecoration for Heading {
        fn thickness(&self, _: &Ui, c: DecorationContent<'_>) -> f32 {
            assert!(matches!(c, DecorationContent::Title("Calibration")));
            32.
        }
        fn show(&self, ui: &mut Ui, area: egui::Rect, _: DecorationContent<'_>, _: Color32) {
            assert!(area.contains_rect(ui.clip_rect()));
            *self.0.lock().unwrap() = Some(area);
        }
    }
    let heading = Arc::new(Heading(Mutex::new(None)));
    let legend = Arc::new(ButtonLegend);
    let path = GeoPathData::from_lat_lon(&[20., 21.], &[45., 46.]).unwrap();
    let polar = PolarCurveData::from_theta_r(&[0., 1.], &[0.5, 1.]).unwrap();
    for geo in [true, false] {
        egui::Context::default()
            .run_ui(support::input(vec![]), |ui| {
                let (plot, entries) = if geo {
                    let p = GeoPlot::new("geo")
                        .title("Calibration")
                        .title_component(heading.clone())
                        .legend_component(legend.clone())
                        .show(
                            ui,
                            &mut Default::default(),
                            &[GeoLine::new(&path).name("calibration interval").into()],
                        );
                    (p.plot_area, p.legend_entries)
                } else {
                    let p = PolarPlot::new("polar")
                        .title("Calibration")
                        .title_component(heading.clone())
                        .legend_component(legend.clone())
                        .show(
                            ui,
                            &mut Default::default(),
                            &[PolarLine::new(&polar).name("calibration interval").into()],
                        );
                    (p.plot_area, p.legend_entries)
                };
                assert!(heading.0.lock().unwrap().unwrap().bottom() <= plot.top());
                assert!(entries[0].rect.left() > plot.right());
            })
            .drop_without_applying_deltas();
    }
}
