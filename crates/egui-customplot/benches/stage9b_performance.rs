//! Headless CPU measurements only: no window, GPU, renderer, or presentation.
use std::{
    hint::black_box,
    mem::size_of,
    time::{Duration, Instant},
};

use egui::Color32;
use egui_customplot::{
    Bounds3D, ColorMap, ColorScale, Contour3D, Contour3DData, ContourLevels, DepthMode,
    ImplicitSurface3DData, Layer3D, MeshSurface3D, Plot3D, Plot3DState, PlotDiagnostics, Point3D,
    Ribbon3D, Ribbon3DData, ScalarField2D, Surface3D, Surface3DData, SurfaceShading, TriMesh3DData,
    TriSurf3D,
};

fn percentile(samples: &mut [f64], q: f64) -> f64 {
    samples.sort_unstable_by(f64::total_cmp);
    samples[((samples.len() - 1) as f64 * q).round() as usize]
}

fn raw_input() -> egui::RawInput {
    egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::vec2(2024.0, 1124.0),
        )),
        ..Default::default()
    }
}

fn bench_preprocess_p50_ms<T>(mut f: impl FnMut() -> T) -> (f64, T) {
    let mut times = Vec::with_capacity(5);
    let mut last = None;
    for _ in 0..5 {
        let start = Instant::now();
        let val = black_box(f());
        times.push(start.elapsed().as_secs_f64() * 1000.0);
        last = Some(val);
    }
    (percentile(&mut times, 0.5), last.unwrap())
}

fn plot3d_primitives(d: &PlotDiagnostics) -> usize {
    d.plot3d.output_triangles + d.plot3d.output_segments + d.plot3d.output_markers
}

fn make_field(n: usize) -> ScalarField2D {
    let nodes: Vec<f64> = (0..n)
        .map(|i| -3.0 + 6.0 * (i as f64 / (n - 1) as f64))
        .collect();
    let mut vals = Vec::with_capacity(n * n);
    for &y in &nodes {
        for &x in &nodes {
            let r2 = x * x + y * y;
            vals.push(1.5 * (-0.28 * r2).exp() * (1.2 * x).cos() + 0.4 * (1.1 * y).sin());
        }
    }
    ScalarField2D::new(nodes.clone(), nodes, vals).unwrap()
}

fn main() {
    let seconds = std::env::var("CUSTOMPLOT_BENCH_SECONDS")
        .ok()
        .and_then(|s| s.parse::<f64>().ok())
        .filter(|s| s.is_finite() && *s > 0.0)
        .unwrap_or(30.0);

    // 1: surf_interp_128x128
    let field_128 = make_field(128);
    let (s128_ms, surf_128) = bench_preprocess_p50_ms(|| Surface3DData::from_field(&field_128));
    let cs_main = ColorScale::new([-1.5, 1.5])
        .unwrap()
        .color_map(ColorMap::Viridis);

    // 2: surfc_surfl_64x64
    let field_64 = make_field(64);
    let (s64_ms, surf_64c) = bench_preprocess_p50_ms(|| {
        Surface3DData::from_field(&field_64)
            .with_contours(ContourLevels::Count(8))
            .unwrap()
    });
    let cs_gray = ColorScale::new([0.0, 1.0])
        .unwrap()
        .color_map(ColorMap::Gray);

    // 3: meshz_hidden_100x100
    let (m100_ms, mesh_100) = bench_preprocess_p50_ms(|| {
        Surface3DData::from_explicit_fn([-2.5, 2.5], [-2.5, 2.5], [100, 100], 20_000, |x, y| {
            0.9 * (x * 0.8).sin() * (y * 0.8).cos()
        })
        .unwrap()
    });

    // 4: waterfall_ribbon_64x32
    let (wr_ms, (wf_surf, rib_8)) = bench_preprocess_p50_ms(|| {
        let wf =
            Surface3DData::from_explicit_fn([-2.0, 2.0], [-2.0, 2.0], [64, 32], 10_000, |x, y| {
                (-0.3 * x * x).exp() * (y * 1.3).cos()
            })
            .unwrap();
        let cols: Vec<Vec<f64>> = (0..8)
            .map(|c| {
                (0..64)
                    .map(|r| ((r as f64 * 0.14) + c as f64 * 0.5).sin() * 0.8)
                    .collect()
            })
            .collect();
        let refs: Vec<&[f64]> = cols.iter().map(Vec::as_slice).collect();
        let rib = Ribbon3DData::new(&refs, None, 0.55).unwrap();
        (wf, rib)
    });

    // 5: contour3_128x128_16levels
    let (c3_ms, contour_16) = bench_preprocess_p50_ms(|| {
        Contour3DData::from_field(&field_128, ContourLevels::Count(16)).unwrap()
    });

    // 6: fimplicit3_48x48x48 (also evaluated end-to-end inside timed frames below)
    let imp_bounds = Bounds3D::new([-2.6; 3], [2.6; 3]).unwrap();
    let (imp_pre_ms, imp_once) = bench_preprocess_p50_ms(|| {
        ImplicitSurface3DData::from_function(imp_bounds, [48, 48, 48], 120_000, |x, y, z| {
            x.cos() + y.cos() + z.cos()
        })
        .unwrap()
    });

    // 7: trisurf_analytic_zbuffer
    let (tri_ms, (sph24, cyl24, ell24, tri_1k)) = bench_preprocess_p50_ms(|| {
        let s = Surface3DData::ellipsoid([-1.8, -0.4, 0.4], [0.7, 0.7, 0.7], 24).unwrap();
        let c = Surface3DData::cylinder(&[0.6, 0.35, 0.75, 0.5], 24).unwrap();
        let e = Surface3DData::ellipsoid([1.8, 0.5, 0.4], [0.85, 0.5, 0.8], 24).unwrap();
        let mut pts: Vec<Point3D> = Vec::with_capacity(1_000);
        for j in 0..25 {
            for i in 0..40 {
                let x = (i as f64 / 39.0) * 3.0 - 1.5;
                let y = (j as f64 / 24.0) * 3.0 + 0.8;
                let z = -0.3 - 0.15 * (x * x + (y - 2.3) * (y - 2.3));
                pts.push([x, y, z]);
            }
        }
        let tm = TriMesh3DData::from_delaunay_xy(&pts).unwrap();
        (s, c, e, tm)
    });

    println!(
        "# Stage 9B CPU; profile=bench; egui=0.37.0; canvas=2024x1124; plot=1920x1080; dpi=1; warmup=5; seconds_per_scene={seconds}"
    );
    println!(
        "# preprocess_p50_ms: surf_interp_128x128={s128_ms:.4}, surfc_surfl_64x64={s64_ms:.4}, meshz_hidden_100x100={m100_ms:.4}, waterfall_ribbon_64x32={wr_ms:.4}, contour3_128x128_16levels={c3_ms:.4}, fimplicit3_48x48x48_extract={imp_pre_ms:.4}, trisurf_analytic_zbuffer={tri_ms:.4}"
    );
    println!(
        "scene,items,frames,seconds,ui_p50_ms,ui_p95_ms,ui_p99_ms,tess_p50_ms,tess_p95_ms,tess_p99_ms,total_p50_ms,total_p95_ms,total_p99_ms,output_primitives,output_meshes,geometry_capacity_bytes,sample_bytes,visited_items"
    );

    measure_plot3d(
        "surf_interp_128x128",
        surf_128.rows() * surf_128.cols(),
        surf_128.sample_bytes(),
        DepthMode::SortedPainter,
        Some((cs_main, "Z")),
        seconds,
        &[Layer3D::from(
            Surface3D::new(&surf_128)
                .shading(SurfaceShading::Interp)
                .color_scale(cs_main),
        )],
    );

    measure_plot3d(
        "surfc_surfl_64x64",
        surf_64c.rows() * surf_64c.cols(),
        surf_64c.sample_bytes(),
        DepthMode::SortedPainter,
        None,
        seconds,
        &[Layer3D::from(
            Surface3D::new(&surf_64c)
                .shading(SurfaceShading::Interp)
                .colormap_lighting(true)
                .color_scale(cs_gray)
                .floor_contours(true)
                .show_normals(true, 0.25)
                .normal_stride(2),
        )],
    );

    measure_plot3d(
        "meshz_hidden_100x100",
        mesh_100.rows() * mesh_100.cols(),
        mesh_100.sample_bytes(),
        DepthMode::SortedPainter,
        None,
        seconds,
        &[Layer3D::from(
            MeshSurface3D::curtain(&mesh_100)
                .hidden_line(true)
                .color_scale(cs_main),
        )],
    );

    measure_plot3d(
        "waterfall_ribbon_64x32",
        wf_surf.rows() * wf_surf.cols() + 8 * 64,
        wf_surf.sample_bytes() + rib_8.sample_bytes(),
        DepthMode::SortedPainter,
        None,
        seconds,
        &[
            Layer3D::from(MeshSurface3D::waterfall(&wf_surf).hidden_line(true)),
            Layer3D::from(Ribbon3D::new(&rib_8).shading(SurfaceShading::Flat)),
        ],
    );

    let c3_pts: usize = contour_16.lines().iter().map(|l| l.points.len()).sum();
    measure_plot3d(
        "contour3_128x128_16levels",
        c3_pts,
        contour_16.sample_bytes(),
        DepthMode::SortedPainter,
        None,
        seconds,
        &[Layer3D::from(
            Contour3D::new(&contour_16).color_scale(cs_main),
        )],
    );

    measure_fimplicit3_e2e(
        "fimplicit3_48x48x48",
        48 * 48 * 48,
        imp_once.sample_bytes() + size_of::<ImplicitSurface3DData>(),
        imp_bounds,
        seconds,
    );

    measure_plot3d(
        "trisurf_analytic_zbuffer",
        sph24.points().len()
            + cyl24.points().len()
            + ell24.points().len()
            + tri_1k.triangles().len(),
        sph24.sample_bytes() + cyl24.sample_bytes() + ell24.sample_bytes() + tri_1k.sample_bytes(),
        DepthMode::ZBuffer,
        None,
        seconds,
        &[
            Layer3D::from(
                Surface3D::new(&sph24)
                    .shading(SurfaceShading::Flat)
                    .color(Color32::from_rgb(65, 145, 220)),
            ),
            Layer3D::from(
                Surface3D::new(&cyl24)
                    .shading(SurfaceShading::Flat)
                    .color(Color32::from_rgb(225, 115, 55)),
            ),
            Layer3D::from(
                Surface3D::new(&ell24)
                    .shading(SurfaceShading::Flat)
                    .color(Color32::from_rgb(65, 175, 115)),
            ),
            Layer3D::from(TriSurf3D::new(&tri_1k).shading(SurfaceShading::Interp)),
        ],
    );
}

fn measure_plot3d(
    scene: &str,
    items: usize,
    sample_bytes: usize,
    depth_mode: DepthMode,
    color_bar: Option<(ColorScale, &str)>,
    seconds: f64,
    layers: &[Layer3D<'_>],
) {
    let ctx = egui::Context::default();
    let mut state = Plot3DState::default();
    let mut ui_times = Vec::new();
    let mut tess_times = Vec::new();
    let mut total_times = Vec::new();
    let mut last_out_prims = 0usize;
    let mut last_visited = 0usize;
    let mut output_meshes = 0usize;
    let mut start_wall = None;
    let mut frame = 0usize;

    while start_wall.is_none_or(|t: Instant| t.elapsed() < Duration::from_secs_f64(seconds)) {
        let start = Instant::now();
        let mut out = ctx.run_ui(raw_input(), |ui| {
            let mut plot = Plot3D::new("stage9b_bench")
                .height(1080.0)
                .depth_mode(depth_mode)
                .show_legend(false)
                .show_hover(false);
            if let Some((cs, lbl)) = color_bar {
                plot = plot.color_bar(cs, lbl);
            }
            let response = plot.show(ui, &mut state, layers);
            assert!(response.plot_area.width() > 0.0 && response.plot_area.height() > 0.0);
            last_out_prims = plot3d_primitives(&response.diagnostics);
            last_visited = response.diagnostics.plot3d.visited_items;
        });
        let ui_elapsed = start.elapsed();
        let meshes = ctx.tessellate(std::mem::take(&mut out.shapes), out.pixels_per_point);
        let total_elapsed = start.elapsed();
        output_meshes = meshes
            .iter()
            .filter(|m| matches!(m.primitive, egui::epaint::Primitive::Mesh(_)))
            .count();
        black_box(&meshes);
        out.drop_without_applying_deltas();
        if frame >= 5 {
            start_wall.get_or_insert_with(Instant::now);
            ui_times.push(ui_elapsed.as_secs_f64() * 1000.0);
            tess_times.push((total_elapsed - ui_elapsed).as_secs_f64() * 1000.0);
            total_times.push(total_elapsed.as_secs_f64() * 1000.0);
        }
        frame += 1;
    }

    print_row(
        scene,
        items,
        start_wall.map_or(0.0, |t| t.elapsed().as_secs_f64()),
        &mut ui_times,
        &mut tess_times,
        &mut total_times,
        last_out_prims,
        output_meshes,
        0,
        sample_bytes,
        last_visited,
    );
}

fn measure_fimplicit3_e2e(
    scene: &str,
    items: usize,
    sample_bytes: usize,
    bounds: Bounds3D,
    seconds: f64,
) {
    let ctx = egui::Context::default();
    let mut state = Plot3DState::default();
    let mut ui_times = Vec::new();
    let mut tess_times = Vec::new();
    let mut total_times = Vec::new();
    let mut last_out_prims = 0usize;
    let mut last_visited = 0usize;
    let mut output_meshes = 0usize;
    let mut start_wall = None;
    let mut frame = 0usize;

    while start_wall.is_none_or(|t: Instant| t.elapsed() < Duration::from_secs_f64(seconds)) {
        let start = Instant::now();
        let mut out = ctx.run_ui(raw_input(), |ui| {
            let imp =
                ImplicitSurface3DData::from_function(bounds, [48, 48, 48], 120_000, |x, y, z| {
                    x.cos() + y.cos() + z.cos()
                })
                .unwrap();
            let response = Plot3D::new("stage9b_imp_e2e")
                .height(1080.0)
                .depth_mode(DepthMode::ZBuffer)
                .show_legend(false)
                .show_hover(false)
                .show(
                    ui,
                    &mut state,
                    &[Layer3D::from(TriSurf3D::from_implicit(&imp))],
                );
            last_out_prims = plot3d_primitives(&response.diagnostics);
            last_visited = response.diagnostics.plot3d.visited_items;
        });
        let ui_elapsed = start.elapsed();
        let meshes = ctx.tessellate(std::mem::take(&mut out.shapes), out.pixels_per_point);
        let total_elapsed = start.elapsed();
        output_meshes = meshes
            .iter()
            .filter(|m| matches!(m.primitive, egui::epaint::Primitive::Mesh(_)))
            .count();
        black_box(&meshes);
        out.drop_without_applying_deltas();
        if frame >= 5 {
            start_wall.get_or_insert_with(Instant::now);
            ui_times.push(ui_elapsed.as_secs_f64() * 1000.0);
            tess_times.push((total_elapsed - ui_elapsed).as_secs_f64() * 1000.0);
            total_times.push(total_elapsed.as_secs_f64() * 1000.0);
        }
        frame += 1;
    }

    print_row(
        scene,
        items,
        start_wall.map_or(0.0, |t| t.elapsed().as_secs_f64()),
        &mut ui_times,
        &mut tess_times,
        &mut total_times,
        last_out_prims,
        output_meshes,
        0,
        sample_bytes,
        last_visited,
    );
}

#[allow(clippy::too_many_arguments)]
fn print_row(
    scene: &str,
    items: usize,
    elapsed_secs: f64,
    ui_times: &mut [f64],
    tess_times: &mut [f64],
    total_times: &mut [f64],
    output_primitives: usize,
    output_meshes: usize,
    geometry_capacity_bytes: usize,
    sample_bytes: usize,
    visited_items: usize,
) {
    let u = [0.5, 0.95, 0.99].map(|q| percentile(ui_times, q));
    let t = [0.5, 0.95, 0.99].map(|q| percentile(tess_times, q));
    let a = [0.5, 0.95, 0.99].map(|q| percentile(total_times, q));
    println!(
        "{scene},{items},{},{elapsed_secs:.3},{:.4},{:.4},{:.4},{:.4},{:.4},{:.4},{:.4},{:.4},{:.4},{output_primitives},{output_meshes},{geometry_capacity_bytes},{sample_bytes},{visited_items}",
        total_times.len(),
        u[0],
        u[1],
        u[2],
        t[0],
        t[1],
        t[2],
        a[0],
        a[1],
        a[2],
    );
}
