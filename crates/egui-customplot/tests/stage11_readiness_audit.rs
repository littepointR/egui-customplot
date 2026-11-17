use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
    time::Duration,
};

use egui::{Context, Pos2, RawInput, Rect, ViewportId};
use egui_customplot::{
    Bar, BarData, Bars, Bounds3D, DepthMode, ErrorBar, ErrorBarData, ErrorBars, Figure,
    GeoBasemapStyle, GeoBounds, GeoBubbleData, GeoBubbles, GeoDensity, GeoDensityData, GeoLayer,
    GeoLine, GeoPathData, GeoPlot, GeoPlotState, GeoScatter, GeoScatterData, Isosurface3D,
    Isosurface3DData, Layer3D, Line, Plot, Plot3D, Plot3DState, PlotData, PlotLayer, PlotState,
    PolarCurveData, PolarHistogram, PolarHistogramData, PolarLayer, PolarLine, PolarPlot,
    PolarState, ScalarVolume3D, Scatter, Surface3D, Surface3DData,
};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[test]
fn checklist_capabilities_egui_perf_and_benchmarks_are_complete_and_traceable() {
    let root = repo_root();
    let checklist = fs::read_to_string(root.join("docs/MATLAB_PLOT_CHECKLIST.md")).unwrap();
    let egui_doc =
        fs::read_to_string(root.join("docs/EGUI_INTEGRATION_AND_PERFORMANCE.md")).unwrap();
    let stage11 = fs::read_to_string(root.join("docs/STAGE_11.md")).unwrap();
    let cargo_toml = fs::read_to_string(root.join("crates/egui-customplot/Cargo.toml")).unwrap();
    let smoke = fs::read_to_string(root.join("scripts/smoke-demo.ps1")).unwrap();

    // 1. All 133 checklist rows exist, are "开发中" (0 "未实现"), and cite an implementation stage.
    let rows: Vec<&str> = checklist
        .lines()
        .filter(|line| {
            ["| ML-", "| EX-", "| LG-"]
                .iter()
                .any(|p| line.starts_with(p))
        })
        .collect();
    assert_eq!(rows.len(), 133, "expected 133 frozen rows");
    for row in &rows {
        let status = row.split('|').nth(2).unwrap().trim();
        assert_eq!(status, "开发中", "expected all 133 rows in 开发中: {row}");
        assert!(
            row.contains("阶段"),
            "every checklist row must cite stage evidence: {row}"
        );
    }

    // 2. CAP-01..20 and TB-01..21 appear in both MATLAB_PLOT_CHECKLIST.md and STAGE_11.md.
    for i in 1..=20 {
        let cap = format!("CAP-{i:02}");
        assert!(checklist.contains(&cap), "checklist missing {cap}");
        assert!(stage11.contains(&cap), "STAGE_11 missing {cap}");
    }
    for i in 1..=21 {
        let tb = format!("TB-{i:02}");
        assert!(checklist.contains(&tb), "checklist missing {tb}");
        assert!(stage11.contains(&tb), "STAGE_11 missing {tb}");
    }

    // 3. EGUI-01..10 and PERF-01..12 appear in both EGUI_INTEGRATION_AND_PERFORMANCE.md and STAGE_11.md.
    for i in 1..=10 {
        let id = format!("EGUI-{i:02}");
        assert!(egui_doc.contains(&id), "EGUI doc missing {id}");
        assert!(stage11.contains(&id), "STAGE_11 missing {id}");
    }
    for i in 1..=12 {
        let id = format!("PERF-{i:02}");
        assert!(egui_doc.contains(&id), "PERF doc missing {id}");
        assert!(stage11.contains(&id), "STAGE_11 missing {id}");
    }

    // 4. Every bench declared in Cargo.toml has a corresponding .rs file under benches/.
    let mut benches = Vec::new();
    let mut next_is_bench = false;
    for line in cargo_toml.lines() {
        let trimmed = line.trim();
        if trimmed == "[[bench]]" {
            next_is_bench = true;
        } else if next_is_bench && trimmed.starts_with("name = ") {
            let name = trimmed
                .strip_prefix("name = ")
                .unwrap()
                .trim()
                .trim_matches('"');
            benches.push(name.to_owned());
            next_is_bench = false;
        }
    }
    assert!(
        benches.len() >= 15,
        "expected >= 15 benchmark targets, got {}",
        benches.len()
    );
    for bench in &benches {
        assert!(
            root.join(format!("crates/egui-customplot/benches/{bench}.rs"))
                .is_file(),
            "missing bench source for {bench}"
        );
    }

    // 5. All 19 demo galleries are listed in smoke-demo.ps1 and benchmark CSV artifacts exist through stage10.
    let expected_galleries = [
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
        "--dynamic-10m",
    ];
    for g in expected_galleries {
        assert!(
            smoke.contains(&format!("'{g}'")),
            "smoke-demo.ps1 missing gallery {g}"
        );
    }

    let bench_csvs: BTreeSet<String> = fs::read_dir(root.join("docs/benchmarks"))
        .unwrap()
        .filter_map(|e| {
            let p = e.unwrap().path();
            (p.extension().is_some_and(|ext| ext == "csv") && fs::metadata(&p).unwrap().len() > 0)
                .then(|| p.file_name().unwrap().to_string_lossy().into_owned())
        })
        .collect();
    for stage_tag in [
        "stage2", "stage4", "stage6", "stage8a", "stage8b", "stage8c", "stage8d", "stage9a",
        "stage9b", "stage9c", "stage10",
    ] {
        assert!(
            bench_csvs.iter().any(|name| name.contains(stage_tag)),
            "missing non-empty benchmark CSV for {stage_tag}"
        );
    }

    // 6. Workspace adopts egui's dual license (MIT OR Apache-2.0) while keeping publish = false.
    assert!(root.join("LICENSE-MIT").is_file(), "missing LICENSE-MIT");
    assert!(
        root.join("LICENSE-APACHE").is_file(),
        "missing LICENSE-APACHE"
    );
    let root_toml = fs::read_to_string(root.join("Cargo.toml")).unwrap();
    assert!(
        root_toml.contains("license = \"MIT OR Apache-2.0\""),
        "workspace license must match egui (MIT OR Apache-2.0)"
    );
    assert!(
        root_toml.contains("publish = false"),
        "workspace publish = false guard must remain intact"
    );
    let lib_toml = fs::read_to_string(root.join("crates/egui-customplot/Cargo.toml")).unwrap();
    let demo_toml = fs::read_to_string(root.join("examples/demo/Cargo.toml")).unwrap();
    assert!(lib_toml.contains("license.workspace = true"));
    assert!(demo_toml.contains("license.workspace = true"));
}

#[test]
fn all_four_coordinate_systems_embed_idempotently_in_one_figure_and_export() {
    // Cell 0: 2D Cartesian (Line + Scatter + Bars + ErrorBars)
    let xy = PlotData::new(vec![[0.0, 1.0], [1.0, 2.4], [2.0, 1.8], [3.0, 3.2]]);
    let bars = BarData::new(vec![Bar::new(0.5, 0.8), Bar::new(2.5, 1.6)]);
    let errs = ErrorBarData::new(vec![ErrorBar::new([1.0, 2.4]).y_error([0.3, 0.4])]);
    let mut s2d = PlotState::default();

    // Cell 1: 2D Polar (PolarLine + PolarHistogram)
    let polar_curve = PolarCurveData::from_theta_r(
        &[0.0, 1.0, 2.0, 3.1, 4.5, 6.0],
        &[1.0, 1.4, 0.8, 1.6, 1.2, 1.0],
    )
    .unwrap();
    let polar_hist = PolarHistogramData::new(&[0.2, 0.3, 1.5, 2.2, 4.0, 5.1], 6).unwrap();
    let mut spolar = PolarState::default();

    // Cell 2: 3D Camera + ZBuffer (Surface3D + Isosurface3D)
    let surf = Surface3DData::from_grid(
        &[-1.0, 0.0, 1.0],
        &[-1.0, 0.0, 1.0],
        &[0.0, 0.4, 0.1, 0.5, 1.0, 0.6, 0.2, 0.5, 0.0],
    )
    .unwrap();
    let vol = ScalarVolume3D::from_function(
        Bounds3D::new([-1.0; 3], [1.0; 3]).unwrap(),
        [10, 10, 10],
        5_000,
        |x, y, z| x * x + y * y + z * z,
    )
    .unwrap();
    let iso = Isosurface3DData::from_volume(&vol, 0.7).unwrap();
    let mut s3d = Plot3DState::default();

    // Cell 3: 2D Geographic (GeoLine + GeoScatter + GeoDensity + GeoBubbles)
    let geo_line =
        GeoPathData::from_lat_lon(&[35.68, 21.31, 37.77], &[139.69, -157.86, -122.42]).unwrap();
    let geo_scat =
        GeoScatterData::from_lat_lon(&[35.68, 21.31, 37.77], &[139.69, -157.86, -122.42]).unwrap();
    let geo_dens = GeoDensityData::from_lat_lon(&[35.68, 36.0, 35.2], &[139.69, 140.0, 139.2])
        .unwrap()
        .radius_km(300.0)
        .unwrap()
        .grid_resolution(24, 18);
    let geo_bub = GeoBubbleData::new(&[35.68, 37.77], &[139.69, -122.42], &[37.0, 4.0]).unwrap();
    let mut sgeo = GeoPlotState::default();

    let ctx = Context::default();
    let raw = |t: f64| RawInput {
        time: Some(t),
        screen_rect: Some(Rect::from_min_size(Pos2::ZERO, egui::vec2(960.0, 680.0))),
        ..Default::default()
    };

    let render_fig = |ui: &mut egui::Ui,
                      s2d: &mut PlotState,
                      spolar: &mut PolarState,
                      s3d: &mut Plot3DState,
                      sgeo: &mut GeoPlotState| {
        egui::ScrollArea::vertical()
            .show(ui, |ui| {
                Figure::new("stage11_four_coords")
                    .columns(2)
                    .cell_height(260.0)
                    .show(ui, 4, |cell| match cell.index() {
                        0 => cell.show_layers(
                            Plot::new("c0").title("Cartesian 2D"),
                            s2d,
                            &[
                                PlotLayer::from(Bars::new(&bars).name("bars")),
                                PlotLayer::from(Line::new(&xy).name("line")),
                                PlotLayer::from(Scatter::new(&xy).name("pts")),
                                PlotLayer::from(ErrorBars::new(&errs).name("err")),
                            ],
                        ),
                        1 => cell
                            .show_polar(
                                PolarPlot::new("c1").title("Polar 2D"),
                                spolar,
                                &[
                                    PolarLayer::from(PolarHistogram::new(&polar_hist).name("rose")),
                                    PolarLayer::from(PolarLine::new(&polar_curve).name("r(theta)")),
                                ],
                            )
                            .into_plot_response(),
                        2 => cell
                            .show_plot3d(
                                Plot3D::new("c2")
                                    .title("Cartesian 3D (ZBuffer)")
                                    .depth_mode(DepthMode::ZBuffer),
                                s3d,
                                &[
                                    Layer3D::from(Surface3D::new(&surf).name("surf")),
                                    Layer3D::from(Isosurface3D::new(&iso).name("iso")),
                                ],
                            )
                            .into_plot_response(),
                        _ => cell
                            .show_geo(
                                GeoPlot::new("c3")
                                    .title("Geographic 2D")
                                    .geo_bounds(GeoBounds::world())
                                    .basemap(GeoBasemapStyle::OfflineVectorLight),
                                sgeo,
                                &[
                                    GeoLayer::from(GeoDensity::new(&geo_dens).name("kde")),
                                    GeoLayer::from(GeoLine::new(&geo_line).name("route")),
                                    GeoLayer::from(GeoScatter::new(&geo_scat).name("stations")),
                                    GeoLayer::from(GeoBubbles::new(&geo_bub).name("hubs")),
                                ],
                            )
                            .into_plot_response(),
                    })
                    .unwrap()
            })
            .inner
    };

    // Frame 0: verify 2-pass layout idempotence across all 4 coordinate systems inside ScrollArea.
    let mut passes = 0usize;
    let out0 = ctx.run_ui(raw(0.0), |ui| {
        let fig = render_fig(ui, &mut s2d, &mut spolar, &mut s3d, &mut sgeo);
        assert_eq!(fig.inner.len(), 4);
        assert!(fig.inner[0].diagnostics.output_paths >= 1);
        assert!(fig.inner[0].diagnostics.output_bars >= 2);
        assert!(fig.inner[1].diagnostics.polar.output_paths >= 1);
        assert!(fig.inner[1].diagnostics.polar.output_sectors >= 1);
        assert!(fig.inner[2].diagnostics.plot3d.output_triangles > 0);
        assert!(fig.inner[3].diagnostics.geo.output_paths >= 1);
        assert!(fig.inner[3].diagnostics.geo.output_bubbles >= 1);
        if passes == 0 {
            ui.ctx().request_discard("stage11 four-coord multipass");
        }
        passes += 1;
    });
    out0.drop_without_applying_deltas();
    assert_eq!(passes, 2);

    // Settled frames: static four-coordinate Figure emits zero immediate repaint request (EGUI-08).
    for step in 1..=6 {
        let out = ctx.run_ui(raw(step as f64 * 0.1), |ui| {
            let _ = render_fig(ui, &mut s2d, &mut spolar, &mut s3d, &mut sgeo);
        });
        if step >= 5 {
            let delay = out.viewport_output[&ViewportId::ROOT].repaint_delay;
            assert!(
                delay > Duration::ZERO,
                "settled 4-coord Figure must not request immediate repaint, got {delay:?}"
            );
        }
        out.drop_without_applying_deltas();
    }

    #[cfg(feature = "export")]
    {
        let mut snap = None;
        ctx.run_ui(raw(1.0), |ui| {
            snap = Some(
                egui_customplot::PlotSnapshot::capture(ui, Default::default(), |ui| {
                    render_fig(ui, &mut s2d, &mut spolar, &mut s3d, &mut sgeo)
                        .response
                        .rect
                })
                .unwrap(),
            );
        })
        .drop_without_applying_deltas();
        drop((
            xy,
            bars,
            errs,
            polar_curve,
            polar_hist,
            surf,
            vol,
            iso,
            geo_line,
            geo_scat,
            geo_dens,
            geo_bub,
            s2d,
            spolar,
            s3d,
            sgeo,
            ctx,
        ));

        let opts = egui_customplot::ExportOptions {
            pixels_per_point: 0.75,
            background: egui::Color32::WHITE,
            ..Default::default()
        };
        let snap = snap.unwrap();
        let svg_bytes = snap.to_svg(opts).unwrap();
        let svg = String::from_utf8(svg_bytes).unwrap();
        assert!(roxmltree::Document::parse(&svg).is_ok());
        assert!(svg.contains("<svg"));
        assert!(!svg.contains("<text"));
        let png = snap.to_png(opts).unwrap();
        assert!(png.starts_with(b"\x89PNG"));
        assert!(png.len() > 2_000);
    }
}
