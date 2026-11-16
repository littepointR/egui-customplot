//! Headless CPU measurements only: no window, GPU, renderer, or presentation.
use std::{
    hint::black_box,
    mem::size_of,
    time::{Duration, Instant},
};

use egui_customplot::{
    Bounds3D, ColorMap, ColorScale, ConePlot3D, ConePlot3DData, Contour3D, Contour3DData,
    ContourLevels, DepthMode, IsocapThreshold, Isocaps3D, Isocaps3DData, Isosurface3D,
    Isosurface3DData, Layer3D, Plot3D, Plot3DState, PlotDiagnostics, Point3D, ScalarVolume3D,
    SlicePlane3D, Stream3DOptions, StreamParticles3D, StreamParticles3DData,
    StreamParticlesPlayback, StreamRibbon3D, StreamRibbon3DData, StreamTube3D, StreamTube3DData,
    TetraMesh3D, TetraMesh3DData, VectorField3D, VolumeSlice3D, VolumeSliceData,
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

fn build_cube_tets(nx: usize, ny: usize, nz: usize) -> TetraMesh3DData {
    let mut verts = Vec::with_capacity((nx + 1) * (ny + 1) * (nz + 1));
    for iz in 0..=nz {
        for iy in 0..=ny {
            for ix in 0..=nx {
                verts.push([
                    -1.0 + 2.0 * (ix as f64 / nx as f64),
                    -1.0 + 2.0 * (iy as f64 / ny as f64),
                    -1.0 + 2.0 * (iz as f64 / nz as f64),
                ]);
            }
        }
    }
    let idx = |x: usize, y: usize, z: usize| ((z * (ny + 1) + y) * (nx + 1) + x) as u32;
    let kuhn = [
        [0, 1, 3, 7],
        [0, 3, 2, 7],
        [0, 2, 6, 7],
        [0, 6, 4, 7],
        [0, 4, 5, 7],
        [0, 5, 1, 7],
    ];
    let mut tets = Vec::with_capacity(nx * ny * nz * 6);
    for z in 0..nz {
        for y in 0..ny {
            for x in 0..nx {
                let c = [
                    idx(x, y, z),
                    idx(x + 1, y, z),
                    idx(x, y + 1, z),
                    idx(x + 1, y + 1, z),
                    idx(x, y, z + 1),
                    idx(x + 1, y, z + 1),
                    idx(x, y + 1, z + 1),
                    idx(x + 1, y + 1, z + 1),
                ];
                for kt in kuhn {
                    tets.push(kt.map(|k| c[k]));
                }
            }
        }
    }
    TetraMesh3DData::new(verts, tets).unwrap()
}

fn main() {
    let seconds = std::env::var("CUSTOMPLOT_BENCH_SECONDS")
        .ok()
        .and_then(|s| s.parse::<f64>().ok())
        .filter(|s| s.is_finite() && *s > 0.0)
        .unwrap_or(30.0);

    let b_sym = Bounds3D::new([-1.5; 3], [1.5; 3]).unwrap();
    let vol64 = ScalarVolume3D::from_function(b_sym, [64, 64, 64], 300_000, |x, y, z| {
        x * x + y * y + z * z
    })
    .unwrap();
    let cs = ColorScale::new([0.0, 6.75])
        .unwrap()
        .color_map(ColorMap::Viridis);

    // 1: slice_3planes_64x64x64
    let (sl_ms, slices3) = bench_preprocess_p50_ms(|| {
        VolumeSliceData::from_volume(
            &vol64,
            &[
                SlicePlane3D::X(0.0),
                SlicePlane3D::Y(0.0),
                SlicePlane3D::Z(0.0),
            ],
        )
        .unwrap()
    });

    // 2: contourslice_4planes_64x64x64
    let (cs_ms, cslice4) = bench_preprocess_p50_ms(|| {
        Contour3DData::from_volume_slices(
            &vol64,
            &[
                SlicePlane3D::X(-0.5),
                SlicePlane3D::X(0.5),
                SlicePlane3D::Y(0.0),
                SlicePlane3D::Z(0.0),
            ],
            ContourLevels::Count(8),
        )
        .unwrap()
    });

    // 3: isosurface_isocaps_48x48x48_zbuffer
    let b_cut = Bounds3D::new([-0.75, -0.75, -0.75], [1.25, 1.25, 1.25]).unwrap();
    let vol48 = ScalarVolume3D::from_function(b_cut, [48, 48, 48], 150_000, |x, y, z| {
        x * x + y * y + z * z
    })
    .unwrap();
    let (iso_ms, (iso48, caps48)) = bench_preprocess_p50_ms(|| {
        let iso = Isosurface3DData::from_volume(&vol48, 1.0).unwrap();
        let caps = Isocaps3DData::from_volume(&vol48, 1.0, IsocapThreshold::Below).unwrap();
        (iso, caps)
    });

    // 4: tetramesh_2k_tets
    let (tet_ms, tets_2k) = bench_preprocess_p50_ms(|| build_cube_tets(7, 7, 7));

    // 5: streamparticles_3d_64lines
    let vf = VectorField3D::from_function(b_sym, [18, 18, 18], 10_000, |x, y, _| {
        [-y + 0.15 * x, x + 0.15 * y, 0.8]
    })
    .unwrap();
    let seeds64: Vec<Point3D> = (0..8)
        .flat_map(|j| {
            (0..8).map(move |i| {
                [
                    -1.1 + 2.2 * (i as f64 / 7.0),
                    -1.1 + 2.2 * (j as f64 / 7.0),
                    -1.2,
                ]
            })
        })
        .collect();
    let opts = Stream3DOptions {
        step_size: 0.06,
        max_steps: 90,
        ..Default::default()
    };
    let (sp_ms, sp_data) = bench_preprocess_p50_ms(|| {
        let sl = egui_customplot::Streamline3DData::from_field(&vf, &seeds64, opts).unwrap();
        StreamParticles3DData::from_streamlines(&sl, 6)
    });

    // 6: streamribbon_streamtube_32seeds
    let seeds16_r = &seeds64[..16];
    let seeds16_t = &seeds64[16..32];
    let (rt_ms, (ribbons, tubes)) = bench_preprocess_p50_ms(|| {
        let r = StreamRibbon3DData::from_field(&vf, seeds16_r, 0.16, 1.0, opts).unwrap();
        let t = StreamTube3DData::from_field(&vf, seeds16_t, 0.10, 8, opts).unwrap();
        (r, t)
    });

    // 7: coneplot_1k_cones_zbuffer
    let pts1k: Vec<Point3D> = (0..10)
        .flat_map(|iz| {
            (0..10).flat_map(move |iy| {
                (0..10).map(move |ix| {
                    [
                        -1.2 + 2.4 * (ix as f64 / 9.0),
                        -1.2 + 2.4 * (iy as f64 / 9.0),
                        -1.2 + 2.4 * (iz as f64 / 9.0),
                    ]
                })
            })
        })
        .collect();
    let (cone_ms, cones_1k) =
        bench_preprocess_p50_ms(|| ConePlot3DData::from_field(&vf, &pts1k, 0.14, 6).unwrap());

    println!(
        "# Stage 9C CPU; profile=bench; egui=0.37.0; canvas=2024x1124; plot=1920x1080; dpi=1; warmup=5; seconds_per_scene={seconds}"
    );
    println!(
        "# preprocess_p50_ms: slice_3planes_64x64x64={sl_ms:.4}, contourslice_4planes_64x64x64={cs_ms:.4}, isosurface_isocaps_48x48x48_zbuffer={iso_ms:.4}, tetramesh_2k_tets={tet_ms:.4}, streamparticles_3d_64lines={sp_ms:.4}, streamribbon_streamtube_32seeds={rt_ms:.4}, coneplot_1k_cones_zbuffer={cone_ms:.4}"
    );
    println!(
        "scene,items,frames,seconds,ui_p50_ms,ui_p95_ms,ui_p99_ms,tess_p50_ms,tess_p95_ms,tess_p99_ms,total_p50_ms,total_p95_ms,total_p99_ms,output_primitives,output_meshes,geometry_capacity_bytes,sample_bytes,visited_items"
    );

    let sl_pts: usize = slices3.slices().iter().map(|s| s.points().len()).sum();
    measure_plot3d(
        "slice_3planes_64x64x64",
        sl_pts,
        slices3.sample_bytes(),
        DepthMode::SortedPainter,
        Some((cs, "V")),
        seconds,
        &[Layer3D::from(VolumeSlice3D::new(&slices3).color_scale(cs))],
    );

    let c_pts: usize = cslice4.lines().iter().map(|l| l.points.len()).sum();
    measure_plot3d(
        "contourslice_4planes_64x64x64",
        c_pts,
        cslice4.sample_bytes(),
        DepthMode::SortedPainter,
        None,
        seconds,
        &[Layer3D::from(Contour3D::new(&cslice4).color_scale(cs))],
    );

    let iso_tris = iso48.mesh().map_or(0, |m| m.triangles().len())
        + caps48.mesh().map_or(0, |m| m.triangles().len());
    measure_plot3d(
        "isosurface_isocaps_48x48x48_zbuffer",
        iso_tris,
        iso48.sample_bytes() + caps48.sample_bytes() + size_of::<Isosurface3DData>(),
        DepthMode::ZBuffer,
        None,
        seconds,
        &[
            Layer3D::from(Isosurface3D::new(&iso48)),
            Layer3D::from(Isocaps3D::new(&caps48).color_scale(cs)),
        ],
    );

    measure_plot3d(
        "tetramesh_2k_tets",
        tets_2k.tetrahedra().len(),
        tets_2k.sample_bytes(),
        DepthMode::SortedPainter,
        None,
        seconds,
        &[Layer3D::from(
            TetraMesh3D::new(&tets_2k).boundary_only(true),
        )],
    );

    measure_particles(
        "streamparticles_3d_64lines",
        sp_data.streamlines().lines().len() * sp_data.particles_per_line(),
        sp_data.sample_bytes(),
        seconds,
        &sp_data,
    );

    let rt_pts: usize = ribbons
        .strips()
        .iter()
        .map(|s| s.points().len())
        .sum::<usize>()
        + tubes
            .tubes()
            .iter()
            .map(|t| t.points().len())
            .sum::<usize>();
    measure_plot3d(
        "streamribbon_streamtube_32seeds",
        rt_pts,
        ribbons.sample_bytes() + tubes.sample_bytes(),
        DepthMode::SortedPainter,
        None,
        seconds,
        &[
            Layer3D::from(StreamRibbon3D::new(&ribbons)),
            Layer3D::from(StreamTube3D::new(&tubes)),
        ],
    );

    measure_plot3d(
        "coneplot_1k_cones_zbuffer",
        cones_1k.cones().len(),
        cones_1k.sample_bytes(),
        DepthMode::ZBuffer,
        None,
        seconds,
        &[Layer3D::from(ConePlot3D::new(&cones_1k))],
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
            let mut plot = Plot3D::new("stage9c_bench")
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

fn measure_particles(
    scene: &str,
    items: usize,
    sample_bytes: usize,
    seconds: f64,
    sp_data: &StreamParticles3DData,
) {
    let ctx = egui::Context::default();
    let mut state = Plot3DState::default();
    let mut playback = StreamParticlesPlayback::new(0.45);
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
            playback.advance_once(ui.ctx().cumulative_frame_nr(), 0.016);
            let response = Plot3D::new("stage9c_particles_bench")
                .height(1080.0)
                .show_legend(false)
                .show_hover(false)
                .show(
                    ui,
                    &mut state,
                    &[Layer3D::from(StreamParticles3D::new(sp_data, &playback))],
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
