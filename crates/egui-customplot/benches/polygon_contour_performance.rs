//! Headless CPU measurements for Stage 8A area/patch/contour/pcolor/image layers.
//! Excludes GPU upload, rasterization, swapchain presentation, and real FPS.
use std::{
    hint::black_box,
    time::{Duration, Instant},
};

use egui_customplot::{
    Area, ColorMap, ColorScale, ContourData, ContourLevels, Contours, FilledContourData,
    FilledContours, ImageData, Patch, PatchData, Pcolor, PcolorShading, Plot, PlotBounds, PlotData,
    PlotDiagnostics, PlotImage, PlotLayer, PlotState, ScalarField2D,
};

fn percentile(samples: &mut [f64], q: f64) -> f64 {
    samples.sort_unstable_by(f64::total_cmp);
    samples[((samples.len() - 1) as f64 * q).round() as usize]
}

fn main() {
    let seconds = std::env::var("CUSTOMPLOT_BENCH_SECONDS")
        .ok()
        .and_then(|s| s.parse::<f64>().ok())
        .filter(|s| s.is_finite() && *s > 0.0)
        .unwrap_or(30.0);

    let field_bounds = PlotBounds::new([0.0, 0.0], [100.0, 100.0]).unwrap();
    let field = ScalarField2D::sample_function(field_bounds, [256, 256], 100_000, |x, y| {
        ((x - 50.0) * 0.12).sin() * ((y - 50.0) * 0.12).cos()
            + 0.4 * (-((x - 50.0).powi(2) + (y - 50.0).powi(2)) * 0.002).exp()
    })
    .unwrap();

    let levels = ContourLevels::Count(12);
    let mut contour_prep = Vec::with_capacity(5);
    let mut contourf_prep = Vec::with_capacity(5);
    let mut contour_data = None;
    let mut contourf_data = None;
    for _ in 0..5 {
        let t0 = Instant::now();
        let c = ContourData::from_field(&field, levels.clone()).unwrap();
        contour_prep.push(t0.elapsed().as_secs_f64() * 1000.0);
        contour_data = Some(c);

        let t1 = Instant::now();
        let cf = FilledContourData::from_field(&field, levels.clone()).unwrap();
        contourf_prep.push(t1.elapsed().as_secs_f64() * 1000.0);
        contourf_data = Some(cf);
    }
    let contour_data = contour_data.unwrap();
    let contourf_data = contourf_data.unwrap();
    let contourf_tris: usize = contourf_data
        .bands()
        .iter()
        .map(|b| b.triangles.len())
        .sum();

    println!(
        "# Stage 8A CPU; profile=bench; egui=0.37.0; canvas=2024x1124; plot=1920x1080; dpi=1; warmup=5; seconds_per_scene={seconds}"
    );
    println!(
        "# preprocess_256x256_12levels_p50_ms: contour={:.4}, contourf={:.4}; contour_lines={}, contourf_bands={}, contourf_tris={contourf_tris}",
        percentile(&mut contour_prep, 0.5),
        percentile(&mut contourf_prep, 0.5),
        contour_data.lines().len(),
        contourf_data.bands().len()
    );
    println!(
        "scene,items,frames,seconds,ui_p50_ms,ui_p95_ms,ui_p99_ms,tess_p50_ms,tess_p95_ms,tess_p99_ms,total_p50_ms,total_p95_ms,total_p99_ms,output_primitives,output_meshes,geometry_capacity_bytes,sample_bytes"
    );

    // 1-2. Indexed 100k Area: full vs narrow 2% X window
    let area_pts: Vec<[f64; 2]> = (0..100_000)
        .map(|i| {
            let x = i as f64 / 999.99;
            let y = 50.0 + 25.0 * (x * 0.25).sin() + 10.0 * (x * 1.1).cos();
            [x, y]
        })
        .collect();
    let mut area_data = PlotData::new(area_pts);
    area_data.build_line_index(16 * 1024 * 1024).unwrap();
    measure(
        "area_100k_full",
        area_data.len(),
        area_data.sample_bytes(),
        field_bounds,
        seconds,
        &[Area::new(&area_data).baseline(50.0).into()],
    );
    measure(
        "area_100k_narrow_2pct",
        area_data.len(),
        area_data.sample_bytes(),
        PlotBounds::new([49.0, 0.0], [51.0, 100.0]).unwrap(),
        seconds,
        &[Area::new(&area_data).baseline(50.0).into()],
    );

    // 3. 100x100 = 10,000 Gouraud quad patch faces
    let side = 100usize;
    let mut p_verts = Vec::with_capacity((side + 1) * (side + 1));
    let mut p_scalars = Vec::with_capacity((side + 1) * (side + 1));
    for r in 0..=side {
        for c in 0..=side {
            let x = c as f64;
            let y = r as f64;
            p_verts.push([x, y]);
            p_scalars.push((x * 0.08).sin() * (y * 0.08).cos());
        }
    }
    let mut p_faces = Vec::with_capacity(side * side);
    for r in 0..side {
        for c in 0..side {
            let i0 = r * (side + 1) + c;
            p_faces.push([i0, i0 + 1, i0 + side + 2, i0 + side + 1]);
        }
    }
    let p_slices: Vec<&[usize]> = p_faces.iter().map(|f| f.as_slice()).collect();
    let patch_data = PatchData::new(p_verts, &p_slices).unwrap();
    let scale = ColorScale::new([-1.0, 1.4])
        .unwrap()
        .color_map(ColorMap::Viridis);
    measure(
        "patch_10k_faces_full",
        patch_data.face_count(),
        patch_data.sample_bytes(),
        field_bounds,
        seconds,
        &[Patch::new(&patch_data)
            .vertex_scalars(&p_scalars, scale)
            .unwrap()
            .into()],
    );

    // 4-6. 256x256 Contours, FilledContours, Pcolor
    measure(
        "contour_256x256_12levels",
        contour_data.lines().len(),
        contour_data.sample_bytes(),
        field_bounds,
        seconds,
        &[Contours::new(&contour_data).color_scale(scale).into()],
    );
    measure(
        "contourf_256x256_12levels",
        contourf_data.bands().len(),
        contourf_data.sample_bytes(),
        field_bounds,
        seconds,
        &[FilledContours::new(&contourf_data, scale).into()],
    );
    measure(
        "pcolor_256x256_flat",
        field.cell_cols() * field.cell_rows(),
        field.sample_bytes(),
        field_bounds,
        seconds,
        &[Pcolor::new(&field, scale)
            .shading(PcolorShading::Flat)
            .into()],
    );

    // 7. 256x256 RGBA8 PlotImage
    let mut rgba = Vec::with_capacity(256 * 256 * 4);
    for r in 0..256usize {
        for c in 0..256usize {
            rgba.extend_from_slice(&[c as u8, r as u8, (c ^ r) as u8, 255]);
        }
    }
    let img_data = ImageData::rgba8(256, 256, field_bounds, rgba).unwrap();
    measure(
        "image_256x256_rgba",
        img_data.width() * img_data.height(),
        img_data.sample_bytes(),
        field_bounds,
        seconds,
        &[PlotImage::new(&img_data).into()],
    );
}

fn measure(
    scene: &str,
    items: usize,
    sample_bytes: usize,
    bounds: PlotBounds,
    seconds: f64,
    layers: &[PlotLayer<'_>],
) {
    let ctx = egui::Context::default();
    let mut state = PlotState::default();
    state.set_bounds(bounds);
    let mut run_frame = || -> (f64, f64, PlotDiagnostics) {
        let start = Instant::now();
        let mut diagnostics = None;
        let mut output = ctx.run_ui(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(2024.0, 1124.0),
                )),
                ..Default::default()
            },
            |ui| {
                let response = Plot::new(scene)
                    .height(1124.0)
                    .show_grid(false)
                    .show_hover(false)
                    .show_layers(ui, &mut state, layers);
                assert_eq!(response.transform.rect().size(), egui::vec2(1920.0, 1080.0));
                diagnostics = Some(response.diagnostics);
            },
        );
        let ui_ms = start.elapsed().as_secs_f64() * 1000.0;
        let start = Instant::now();
        let meshes = ctx.tessellate(std::mem::take(&mut output.shapes), output.pixels_per_point);
        let tess_ms = start.elapsed().as_secs_f64() * 1000.0;
        black_box(meshes);
        output.drop_without_applying_deltas();
        (ui_ms, tess_ms, diagnostics.unwrap())
    };
    for _ in 0..5 {
        black_box(run_frame());
    }
    let start = Instant::now();
    let mut ui = Vec::new();
    let mut tess = Vec::new();
    let mut total = Vec::new();
    let mut last;
    loop {
        let (a, b, d) = run_frame();
        ui.push(a);
        tess.push(b);
        total.push(a + b);
        last = d;
        if start.elapsed() >= Duration::from_secs_f64(seconds) {
            break;
        }
    }
    let elapsed = start.elapsed().as_secs_f64();
    let frames = ui.len();
    let values = [
        percentile(&mut ui, 0.5),
        percentile(&mut ui, 0.95),
        percentile(&mut ui, 0.99),
        percentile(&mut tess, 0.5),
        percentile(&mut tess, 0.95),
        percentile(&mut tess, 0.99),
        percentile(&mut total, 0.5),
        percentile(&mut total, 0.95),
        percentile(&mut total, 0.99),
    ];
    let output_primitives = last.patches.output_triangles
        + last.contours.output_paths
        + last.contours.visited_items
        + last.images.output_cells;
    let output_meshes =
        last.patches.output_meshes + last.contours.output_meshes + last.images.output_meshes;
    println!(
        "{scene},{items},{frames},{elapsed:.3},{:.4},{:.4},{:.4},{:.4},{:.4},{:.4},{:.4},{:.4},{:.4},{output_primitives},{output_meshes},{},{}",
        values[0],
        values[1],
        values[2],
        values[3],
        values[4],
        values[5],
        values[6],
        values[7],
        values[8],
        last.geometry_capacity_bytes,
        sample_bytes
    );
}
