use std::{f64::consts::TAU, fs, path::PathBuf};

use egui::{Color32, Context, Event, PointerButton, Pos2, RawInput, Rect, pos2};
use egui_customplot::{
    Bounds3D, ColorMap, DepthMode, GanttChart, GanttData, GanttTask, GeoGlobeData, GeoGlobePlot,
    GeoGlobeState, GeoGlobeTransform, GlobeMarker3D, Layer3D, ModernError, Plot, Plot3D,
    Plot3DState, PlotLayer, PlotState, PointCloud3D, PointCloud3DData, RadarChart, RadarData,
    RegionKind, SankeyChart, SankeyData, ScalarVolume3D, SunburstChart, SunburstData,
    SunburstInput, TreemapChart, TreemapData, VolumeRender3D, VolumeRenderMode,
    VolumeTransferFunction,
};

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
}

fn raw() -> RawInput {
    RawInput {
        screen_rect: Some(Rect::from_min_size(Pos2::ZERO, egui::vec2(640.0, 480.0))),
        ..Default::default()
    }
}

fn hovered_raw(pos: Pos2) -> RawInput {
    let mut r = raw();
    r.events.push(Event::PointerMoved(pos));
    r
}

#[test]
fn ext23_volshow_modes_zbuffer_occlusion_and_hover_work() {
    let bounds = Bounds3D::new([-1.0, -1.0, -1.0], [1.0, 1.0, 1.0]).unwrap();
    let vol = ScalarVolume3D::from_function(bounds, [20, 20, 20], 20_000, |x, y, z| {
        (-(x * x + y * y + z * z) * 2.5).exp()
    })
    .unwrap();
    let tf = VolumeTransferFunction::ramp(ColorMap::Inferno, 0.0, 0.9);

    for mode in [
        VolumeRenderMode::DirectAlpha,
        VolumeRenderMode::MaximumIntensity,
        VolumeRenderMode::IsosurfaceRay { threshold: 0.45 },
    ] {
        let ctx = Context::default();
        let mut state = Plot3DState::default();
        let mut center = Pos2::ZERO;
        let out = ctx.run_ui(raw(), |ui| {
            let resp = Plot3D::new("vol_mode")
                .height(360.0)
                .depth_mode(DepthMode::ZBuffer)
                .show(
                    ui,
                    &mut state,
                    &[Layer3D::from(
                        VolumeRender3D::new(&vol)
                            .transfer(tf)
                            .mode(mode)
                            .name("HeartVol"),
                    )],
                );
            assert!(
                resp.diagnostics.plot3d.zbuffer_pixels > 0,
                "expected non-empty ZBuffer for {mode:?}"
            );
            center = resp.plot_area.center();
        });
        assert!(!out.shapes.is_empty());
        out.drop_without_applying_deltas();

        ctx.run_ui(hovered_raw(center), |ui| {
            let resp = Plot3D::new("vol_mode")
                .height(360.0)
                .depth_mode(DepthMode::ZBuffer)
                .show(
                    ui,
                    &mut state,
                    &[Layer3D::from(
                        VolumeRender3D::new(&vol)
                            .transfer(tf)
                            .mode(mode)
                            .name("HeartVol"),
                    )],
                );
            let hr = resp.hovered_region.expect("expected volume raycast hover");
            assert!(hr.label.contains("HeartVol") && hr.label.contains("peak="));
        })
        .drop_without_applying_deltas();
    }
}

#[test]
fn ext24_pcshow_200k_octree_lod_and_nearest_picking_work() {
    let n = 200_000usize;
    let mut pts = Vec::with_capacity(n);
    let mut intens = Vec::with_capacity(n);
    for i in 0..n {
        let t = i as f64 / n as f64;
        let a = 40.0 * TAU * t;
        let r = 0.1 + 0.9 * t;
        pts.push([r * a.cos(), r * a.sin(), 2.0 * t - 1.0]);
        intens.push(t as f32);
    }
    let cloud = PointCloud3DData::from_xyz(pts.clone())
        .unwrap()
        .with_intensity(intens)
        .unwrap();

    let ctx = Context::default();
    let mut state = Plot3DState::default();
    let mut pick_pos = Pos2::ZERO;
    ctx.run_ui(raw(), |ui| {
        let resp = Plot3D::new("pc_200k")
            .height(360.0)
            .depth_mode(DepthMode::ZBuffer)
            .show(
                ui,
                &mut state,
                &[Layer3D::from(
                    PointCloud3D::new(&cloud)
                        .max_points(6_000)
                        .name("Cloud200k"),
                )],
            );
        assert!(
            resp.diagnostics.plot3d.visited_items > 0
                && resp.diagnostics.plot3d.visited_items <= 6_000,
            "octree LOD must bound visited markers (got {})",
            resp.diagnostics.plot3d.visited_items
        );
        pick_pos = resp.transform.project_data(pts[0]).unwrap().0;
    })
    .drop_without_applying_deltas();

    ctx.run_ui(hovered_raw(pick_pos), |ui| {
        let resp = Plot3D::new("pc_200k")
            .height(360.0)
            .depth_mode(DepthMode::ZBuffer)
            .show(
                ui,
                &mut state,
                &[Layer3D::from(
                    PointCloud3D::new(&cloud)
                        .max_points(6_000)
                        .name("Cloud200k"),
                )],
            );
        assert!(resp.hovered_point.is_some(), "expected nearest octree pick");
    })
    .drop_without_applying_deltas();
}

#[test]
fn ext25_geoglobe_projection_backface_culling_orbit_and_multipass() {
    let tf = GeoGlobeTransform::new(
        Rect::from_min_size(Pos2::ZERO, egui::vec2(500.0, 500.0)),
        30.0,
        120.0,
        1.0,
        1.0,
    );
    let (front, pz) = tf.project(30.0, 120.0, 0.0).expect("front center");
    assert!((front.x - 250.0).abs() < 1e-3 && (front.y - 250.0).abs() < 1e-3);
    assert!(pz > 0.99);
    assert!(
        tf.project(-30.0, -60.0, 0.0).is_none(),
        "antipodal point must be backface-culled"
    );
    let back_ll = tf.screen_to_lat_lon(front).expect("inverse inside disk");
    assert!((back_ll[0] - 30.0).abs() < 1e-2 && (back_ll[1] - 120.0).abs() < 1e-2);

    let globe = GeoGlobeData::new()
        .with_extruded_track(
            "Orbit",
            &[25.0, 30.0, 35.0],
            &[105.0, 112.0, 121.5],
            &[100_000.0, 400_000.0, 100_000.0],
            Color32::YELLOW,
        )
        .unwrap()
        .with_markers(vec![GlobeMarker3D {
            label: "Hub".into(),
            lat_deg: 25.0,
            lon_deg: 105.0,
            alt_m: 0.0,
            radius_px: 5.0,
            color: Color32::CYAN,
        }])
        .unwrap();

    let ctx = Context::default();
    let mut state = GeoGlobeState::default();
    let mut hub_pos = Pos2::ZERO;
    ctx.run_ui(raw(), |ui| {
        let resp = GeoGlobePlot::new("g25", &globe)
            .height(340.0)
            .show(ui, &mut state);
        hub_pos = resp.transform.project(25.0, 105.0, 0.0).unwrap().0;
    })
    .drop_without_applying_deltas();

    ctx.run_ui(hovered_raw(hub_pos), |ui| {
        let resp = GeoGlobePlot::new("g25", &globe)
            .height(340.0)
            .show(ui, &mut state);
        assert_eq!(resp.hovered_item.unwrap().label, "Hub");
    })
    .drop_without_applying_deltas();

    // Two-pass orbit drag must apply navigation once per frame.
    let mut move_start = raw();
    move_start.events.push(Event::PointerMoved(hub_pos));
    ctx.run_ui(move_start, |ui| {
        let _ = GeoGlobePlot::new("g25", &globe)
            .height(340.0)
            .show(ui, &mut state);
    })
    .drop_without_applying_deltas();

    let mut down = raw();
    down.events.push(Event::PointerButton {
        pos: hub_pos,
        button: PointerButton::Primary,
        pressed: true,
        modifiers: Default::default(),
    });
    ctx.run_ui(down, |ui| {
        let _ = GeoGlobePlot::new("g25", &globe)
            .height(340.0)
            .show(ui, &mut state);
    })
    .drop_without_applying_deltas();

    let (lat_before, lon_before) = state.center_lat_lon();
    let mut drag = raw();
    drag.events.push(Event::PointerMoved(pos2(
        hub_pos.x + 28.0,
        hub_pos.y + 12.0,
    )));
    let mut passes = 0;
    let mut single_pass_lon = None;
    ctx.run_ui(drag, |ui| {
        let _ = GeoGlobePlot::new("g25", &globe)
            .height(340.0)
            .show(ui, &mut state);
        if passes == 0 {
            single_pass_lon = Some(state.center_lat_lon());
            ui.ctx().request_discard("globe multipass drag guard");
        }
        passes += 1;
    })
    .drop_without_applying_deltas();
    assert_eq!(passes, 2);
    assert_ne!((lat_before, lon_before), state.center_lat_lon());
    assert_eq!(single_pass_lon.unwrap(), state.center_lat_lon());
}

#[test]
fn ext26_modern_charts_invariants_and_picking_work() {
    assert_eq!(
        RadarData::new(&["A", "B"], &[1.0, 1.0], &[("S", &[1.0, 1.0])]).unwrap_err(),
        ModernError::Empty
    );
    assert_eq!(
        SankeyData::new(&["A", "B"], &[(0, 0, 1.0)]).unwrap_err(),
        ModernError::InvalidNode
    );
    assert_eq!(
        GanttData::new(
            vec![GanttTask {
                label: "bad".into(),
                start: 5.0,
                end: 2.0,
                progress: 0.5,
                group: 0,
                milestone: false,
            }],
            vec![],
        )
        .unwrap_err(),
        ModernError::InvalidInterval
    );

    let radar = RadarData::new(
        &["A", "B", "C", "D"],
        &[10.0; 4],
        &[("S", &[8.0, 6.0, 9.0, 7.0])],
    )
    .unwrap();
    let sankey = SankeyData::new(
        &["Src1", "Src2", "Mid", "Out"],
        &[(0, 2, 30.0), (1, 2, 20.0), (2, 3, 50.0)],
    )
    .unwrap();
    let treemap =
        TreemapData::squarified(&[("X", 0, 50.0), ("Y", 1, 30.0), ("Z", 2, 20.0)]).unwrap();
    let total_area: f64 = treemap
        .rects()
        .iter()
        .map(|r| (r.x1 - r.x0) * (r.y1 - r.y0))
        .sum();
    assert!((total_area - 1.0).abs() < 1e-10);

    let sunburst = SunburstData::new(&[
        SunburstInput {
            label: "Root".into(),
            parent: None,
            value: 0.0,
            group: 0,
        },
        SunburstInput {
            label: "C1".into(),
            parent: Some(0),
            value: 3.0,
            group: 1,
        },
        SunburstInput {
            label: "C2".into(),
            parent: Some(0),
            value: 7.0,
            group: 2,
        },
    ])
    .unwrap();
    let child_span: f64 = sunburst
        .sectors()
        .iter()
        .filter(|s| s.depth == 1)
        .map(|s| s.end_rad - s.start_rad)
        .sum();
    assert!((child_span - TAU).abs() < 1e-10);

    let gantt = GanttData::new(
        vec![GanttTask {
            label: "Task 1".into(),
            start: 1.0,
            end: 5.0,
            progress: 0.75,
            group: 0,
            milestone: false,
        }],
        vec![],
    )
    .unwrap();

    // Verify region & point hover on modern layers (each isolated in its own 2-frame context).
    {
        let ctx = Context::default();
        let mut st = PlotState::default();
        let mut p = Pos2::ZERO;
        ctx.run_ui(raw(), |ui| {
            let r = Plot::new("r0").canvas_axes().show_layers(
                ui,
                &mut st,
                &[PlotLayer::from(RadarChart::new(&radar))],
            );
            p = r
                .transform
                .position_from_point(radar.series()[0].vertices[0]);
        })
        .drop_without_applying_deltas();
        ctx.run_ui(hovered_raw(p), |ui| {
            let r = Plot::new("r0").canvas_axes().show_layers(
                ui,
                &mut st,
                &[PlotLayer::from(RadarChart::new(&radar))],
            );
            assert_eq!(r.hovered_point.unwrap().point_index, 0);
        })
        .drop_without_applying_deltas();
    }
    {
        let ctx = Context::default();
        let mut st = PlotState::default();
        let mut p = Pos2::ZERO;
        ctx.run_ui(raw(), |ui| {
            let r = Plot::new("s1").canvas_axes().show_layers(
                ui,
                &mut st,
                &[PlotLayer::from(SankeyChart::new(&sankey))],
            );
            let rib = &sankey.ribbons()[0];
            let mid_idx = rib.top_curve.len() / 2;
            p = r.transform.position_from_point([
                rib.top_curve[mid_idx][0],
                0.5 * (rib.top_curve[mid_idx][1] + rib.bottom_curve[mid_idx][1]),
            ]);
        })
        .drop_without_applying_deltas();
        ctx.run_ui(hovered_raw(p), |ui| {
            let r = Plot::new("s1").canvas_axes().show_layers(
                ui,
                &mut st,
                &[PlotLayer::from(SankeyChart::new(&sankey))],
            );
            assert_eq!(r.hovered_region.unwrap().kind, RegionKind::SankeyRibbon);
        })
        .drop_without_applying_deltas();
    }
    {
        let ctx = Context::default();
        let mut st = PlotState::default();
        let mut p = Pos2::ZERO;
        ctx.run_ui(raw(), |ui| {
            let r = Plot::new("t2").canvas_axes().show_layers(
                ui,
                &mut st,
                &[PlotLayer::from(TreemapChart::new(&treemap))],
            );
            let tr = &treemap.rects()[0];
            p = r
                .transform
                .position_from_point([0.5 * (tr.x0 + tr.x1), 0.5 * (tr.y0 + tr.y1)]);
        })
        .drop_without_applying_deltas();
        ctx.run_ui(hovered_raw(p), |ui| {
            let r = Plot::new("t2").canvas_axes().show_layers(
                ui,
                &mut st,
                &[PlotLayer::from(TreemapChart::new(&treemap))],
            );
            assert_eq!(r.hovered_region.unwrap().kind, RegionKind::TreemapCell);
        })
        .drop_without_applying_deltas();
    }
    {
        let ctx = Context::default();
        let mut st = PlotState::default();
        let mut p = Pos2::ZERO;
        ctx.run_ui(raw(), |ui| {
            let r = Plot::new("u3").canvas_axes().show_layers(
                ui,
                &mut st,
                &[PlotLayer::from(SunburstChart::new(&sunburst))],
            );
            let sec = &sunburst.sectors()[1];
            let rm = 0.5 * (sec.r_in + sec.r_out);
            let am = 0.5 * (sec.start_rad + sec.end_rad);
            p = r
                .transform
                .position_from_point([rm * am.cos(), rm * am.sin()]);
        })
        .drop_without_applying_deltas();
        ctx.run_ui(hovered_raw(p), |ui| {
            let r = Plot::new("u3").canvas_axes().show_layers(
                ui,
                &mut st,
                &[PlotLayer::from(SunburstChart::new(&sunburst))],
            );
            assert_eq!(r.hovered_region.unwrap().kind, RegionKind::SunburstSector);
        })
        .drop_without_applying_deltas();
    }
    {
        let ctx = Context::default();
        let mut st = PlotState::default();
        let mut p = Pos2::ZERO;
        ctx.run_ui(raw(), |ui| {
            let r = Plot::new("g4").gantt_axes(&gantt).show_layers(
                ui,
                &mut st,
                &[PlotLayer::from(GanttChart::new(&gantt))],
            );
            p = r.transform.position_from_point([3.0, 1.0]);
        })
        .drop_without_applying_deltas();
        ctx.run_ui(hovered_raw(p), |ui| {
            let r = Plot::new("g4").gantt_axes(&gantt).show_layers(
                ui,
                &mut st,
                &[PlotLayer::from(GanttChart::new(&gantt))],
            );
            assert_eq!(r.hovered_region.unwrap().kind, RegionKind::GanttTaskBar);
        })
        .drop_without_applying_deltas();
    }
}

#[test]
fn stage16_full_expansion_plan_and_checklist_audit() {
    let root = repo_root();
    let baseline = fs::read_to_string(root.join("docs/MATLAB_PLOT_BASELINE.json")).unwrap();
    let checklist = fs::read_to_string(root.join("docs/MATLAB_PLOT_CHECKLIST.md")).unwrap();
    assert!(baseline.contains("\"unique_function_count\": 82"));
    assert!(checklist.contains("82 项 ML、35 项 EX 与 16 项 LG 共 133 项"));

    let plan = fs::read_to_string(root.join("docs/EXPANSION_PLAN.md")).unwrap();
    for id in 1..=26 {
        let code = format!("`EXT-{id:02}`");
        assert!(plan.contains(&code), "EXPANSION_PLAN.md missing {code}");
    }
    assert!(
        !plan.contains("| 未实现 |"),
        "EXPANSION_PLAN.md still contains unimplemented rows"
    );

    for doc in [
        "docs/STAGE_12.md",
        "docs/STAGE_13.md",
        "docs/STAGE_14.md",
        "docs/STAGE_15.md",
        "docs/STAGE_16.md",
    ] {
        let p = root.join(doc);
        assert!(
            p.is_file() && fs::metadata(&p).unwrap().len() > 0,
            "missing {doc}"
        );
    }

    let smoke = fs::read_to_string(root.join("scripts/smoke-demo.ps1")).unwrap();
    for g in [
        "default",
        "--scalar",
        "--statistical",
        "--stairs",
        "--stems",
        "--quiver",
        "--polygons",
        "--contours",
        "--functions",
        "--geometry",
        "--distributions",
        "--multivariate",
        "--polar",
        "--networks",
        "--plot3d",
        "--surfaces3d",
        "--volumes3d",
        "--geo",
        "--stage13",
        "--stage14",
        "--stage15",
        "--stage16",
        "--dynamic-10m",
    ] {
        assert!(
            smoke.contains(&format!("'{g}'")),
            "smoke-demo.ps1 missing {g}"
        );
    }

    let lib_toml = fs::read_to_string(root.join("crates/egui-customplot/Cargo.toml")).unwrap();
    assert!(lib_toml.contains("stage16_performance"));
    assert!(
        root.join("crates/egui-customplot/benches/stage16_performance.rs")
            .is_file()
    );
}
