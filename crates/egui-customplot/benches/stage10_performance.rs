//! Headless CPU measurements only: no window, GPU, renderer, or presentation.
use std::{
    hint::black_box,
    mem::{size_of, size_of_val},
    time::{Duration, Instant},
};

use egui_customplot::{
    ColorMap, ColorScale, GeoAttribution, GeoBasemapStyle, GeoBounds, GeoBubbleData, GeoBubbles,
    GeoDensity, GeoDensityData, GeoLayer, GeoLine, GeoOfflinePolicy, GeoPathData, GeoPlot,
    GeoPlotState, GeoPoint, GeoRasterTile, GeoScatter, GeoScatterData, GeoTileCache, GeoTileKey,
    PlotDiagnostics,
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

fn geo_primitives(d: &PlotDiagnostics) -> usize {
    d.geo.output_paths
        + d.geo.output_markers
        + d.geo.output_bubbles
        + d.geo.output_density_cells
        + d.geo.output_basemap_polygons
}

fn main() {
    let seconds: f64 = std::env::args()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(30.0);

    // 1: 50k antimeridian-crossing routes
    let mut r_lat = Vec::with_capacity(50_000);
    let mut r_lon = Vec::with_capacity(50_000);
    for i in 0..50_000 {
        let t = i as f64 * 0.015;
        r_lat.push((t * 0.13).sin() * 52.0);
        r_lon.push(-180.0 + ((i as f64 * 1.43) % 360.0));
    }
    let (route_ms, route_50k) =
        bench_preprocess_p50_ms(|| GeoPathData::from_lat_lon(&r_lat, &r_lon).unwrap());

    // 2: 20k per-point styled scatter
    let mut s_lat = Vec::with_capacity(20_000);
    let mut s_lon = Vec::with_capacity(20_000);
    let mut s_sz = Vec::with_capacity(20_000);
    let mut s_col = Vec::with_capacity(20_000);
    for i in 0..20_000 {
        let f = i as f64;
        s_lat.push(((f * 0.37).sin()) * 65.0);
        s_lon.push(((f * 0.61).cos()) * 175.0);
        s_sz.push(2.0 + (i % 5) as f32 * 0.8);
        s_col.push((f * 0.01).sin() * 50.0 + 50.0);
    }
    let cs_scatter = ColorScale::new([0.0, 100.0])
        .unwrap()
        .color_map(ColorMap::Viridis);
    let (scatter_ms, scatter_20k) = bench_preprocess_p50_ms(|| {
        GeoScatterData::from_lat_lon(&s_lat, &s_lon)
            .unwrap()
            .with_sizes(&s_sz)
            .unwrap()
            .with_color_values(&s_col)
            .unwrap()
    });

    // 3: 10k weighted density on 64x48 grid
    let mut d_lat = Vec::with_capacity(10_000);
    let mut d_lon = Vec::with_capacity(10_000);
    let mut d_w = Vec::with_capacity(10_000);
    for i in 0..10_000 {
        let f = i as f64;
        let (cl, cln) = match i % 4 {
            0 => (48.0, 9.0),
            1 => (38.0, -96.0),
            2 => (35.0, 117.0),
            _ => (-25.0, 134.0),
        };
        d_lat.push(cl + (f * 0.19).sin() * 8.0);
        d_lon.push(cln + (f * 0.29).cos() * 14.0);
        d_w.push(0.5 + (i % 5) as f64 * 0.4);
    }
    let cs_density = ColorScale::new([0.0, 200.0])
        .unwrap()
        .color_map(ColorMap::BlueRed);
    let (density_ms, density_10k) = bench_preprocess_p50_ms(|| {
        GeoDensityData::weighted(&d_lat, &d_lon, &d_w)
            .unwrap()
            .radius_km(350.0)
            .unwrap()
            .grid_resolution(64, 48)
    });

    // 4: 2k categorical bubbles + BubbleLegend
    let mut b_lat = Vec::with_capacity(2_000);
    let mut b_lon = Vec::with_capacity(2_000);
    let mut b_sz = Vec::with_capacity(2_000);
    let mut b_cat = Vec::with_capacity(2_000);
    let cat_names = ["APAC", "EMEA", "AMER", "LATAM"];
    for i in 0..2_000 {
        let f = i as f64;
        b_lat.push((f * 0.41).sin() * 58.0);
        b_lon.push((f * 0.73).cos() * 170.0);
        b_sz.push(5.0 + (i % 40) as f64 * 2.5);
        b_cat.push(cat_names[i % 4]);
    }
    let (bubble_ms, bubble_2k) = bench_preprocess_p50_ms(|| {
        GeoBubbleData::new(&b_lat, &b_lon, &b_sz)
            .unwrap()
            .with_categories(&b_cat)
            .unwrap()
    });
    let bubble_legend = bubble_2k.bubble_legend("Scale").steps(3);

    // 5: HostTiles cache population
    let (tiles_ms, mut tile_cache) = bench_preprocess_p50_ms(|| {
        let mut cache = GeoTileCache::new(
            256 * 1024,
            GeoAttribution::new("Bench synthetic tiles").with_license("CC0-1.0"),
        )
        .unwrap()
        .with_offline_policy(GeoOfflinePolicy::FallbackToVector);
        let tile = GeoRasterTile::from_rgba(32, 32, [190, 220, 240, 255].repeat(32 * 32)).unwrap();
        for y in 0..4 {
            for x in 0..4 {
                if (x + y) % 3 != 0 {
                    cache
                        .insert(GeoTileKey::new(2, x, y).unwrap(), tile.clone())
                        .unwrap();
                }
            }
        }
        cache
    });

    // 6: Pacific 4-layer combo preprocessing time (small route slice cost check)
    let pacific_route = GeoPathData::from_lat_lon(
        &[35.68, 21.31, 37.77, -33.87, -36.85],
        &[139.69, -157.86, -122.42, 151.21, 174.76],
    )
    .unwrap();
    let (combo_ms, pacific_bounds) = bench_preprocess_p50_ms(|| {
        GeoBounds::from_lat_lon([-50.0, 55.0], [120.0, -110.0]).unwrap()
    });

    println!(
        "# Stage 10 CPU; profile=bench; egui=0.37.0; canvas=2024x1124; plot=1920x1080; dpi=1; warmup=5; seconds_per_scene={seconds}"
    );
    println!(
        "# preprocess_p50_ms: geoplot_routes_50k_dateline={route_ms:.4}, geoscatter_20k_styled={scatter_ms:.4}, geodensity_10k_weighted_64x48={density_ms:.4}, geobubble_2k_categorical_legend={bubble_ms:.4}, geobasemap_host_tiles_cached={tiles_ms:.4}, geo_combo_pacific_4layers={combo_ms:.4}"
    );
    println!(
        "scene,items,frames,seconds,ui_p50_ms,ui_p95_ms,ui_p99_ms,tess_p50_ms,tess_p95_ms,tess_p99_ms,total_p50_ms,total_p95_ms,total_p99_ms,output_primitives,output_meshes,geometry_capacity_bytes,sample_bytes,visited_items"
    );

    measure_geo(
        "geoplot_routes_50k_dateline",
        50_000,
        size_of_val(route_50k.points()),
        seconds,
        GeoBounds::world(),
        GeoBasemapStyle::OfflineVectorLight,
        None,
        None,
        &[GeoLayer::from(GeoLine::new(&route_50k))],
        None,
    );

    measure_geo(
        "geoscatter_20k_styled",
        20_000,
        size_of_val(scatter_20k.points()),
        seconds,
        GeoBounds::world(),
        GeoBasemapStyle::OfflineVectorLight,
        None,
        None,
        &[GeoLayer::from(
            GeoScatter::new(&scatter_20k).color_scale(cs_scatter),
        )],
        None,
    );

    measure_geo(
        "geodensity_10k_weighted_64x48",
        10_000,
        10_000 * (size_of::<GeoPoint>() + size_of::<f64>()),
        seconds,
        GeoBounds::world(),
        GeoBasemapStyle::OfflineVectorDark,
        Some((cs_density, "Density")),
        None,
        &[GeoLayer::from(
            GeoDensity::new(&density_10k).color_scale(cs_density),
        )],
        None,
    );

    measure_geo(
        "geobubble_2k_categorical_legend",
        2_000,
        size_of_val(bubble_2k.points()),
        seconds,
        GeoBounds::world(),
        GeoBasemapStyle::OfflineVectorLight,
        None,
        Some(bubble_legend.clone()),
        &[GeoLayer::from(GeoBubbles::new(&bubble_2k))],
        None,
    );

    let tile_bytes = tile_cache.retained_bytes();
    measure_geo(
        "geobasemap_host_tiles_cached",
        tile_cache.len(),
        tile_bytes,
        seconds,
        GeoBounds::from_lat_lon([-60.0, 65.0], [-140.0, 140.0]).unwrap(),
        GeoBasemapStyle::HostTiles,
        None,
        None,
        &[GeoLayer::from(GeoLine::new(&pacific_route))],
        Some(&mut tile_cache),
    );

    let combo_bytes = size_of_val(pacific_route.points())
        + size_of_val(scatter_20k.points())
        + 10_000 * (size_of::<GeoPoint>() + size_of::<f64>())
        + size_of_val(bubble_2k.points());
    measure_geo(
        "geo_combo_pacific_4layers",
        pacific_route.points().len() + 20_000 + 10_000 + 2_000,
        combo_bytes,
        seconds,
        pacific_bounds,
        GeoBasemapStyle::OfflineVectorLight,
        None,
        Some(bubble_legend),
        &[
            GeoLayer::from(GeoDensity::new(&density_10k).color_scale(cs_density)),
            GeoLayer::from(GeoLine::new(&pacific_route)),
            GeoLayer::from(GeoScatter::new(&scatter_20k).color_scale(cs_scatter)),
            GeoLayer::from(GeoBubbles::new(&bubble_2k)),
        ],
        None,
    );
}

#[allow(clippy::too_many_arguments)]
fn measure_geo(
    scene: &str,
    items: usize,
    sample_bytes: usize,
    seconds: f64,
    bounds: GeoBounds,
    basemap: GeoBasemapStyle,
    color_bar: Option<(ColorScale, &'static str)>,
    bubble_legend: Option<egui_customplot::BubbleLegend>,
    layers: &[GeoLayer<'_>],
    mut tile_cache: Option<&mut GeoTileCache>,
) {
    let ctx = egui::Context::default();
    let mut state = GeoPlotState::default();
    let (mut ui_times, mut tess_times, mut total_times) = (Vec::new(), Vec::new(), Vec::new());
    let (mut last_prims, mut last_visited, mut out_meshes) = (0, 0, 0);
    let mut start_wall = None;
    let mut frame = 0usize;

    while start_wall.is_none_or(|t: Instant| t.elapsed() < Duration::from_secs_f64(seconds)) {
        let start = Instant::now();
        let mut out = ctx.run_ui(raw_input(), |ui| {
            let mut plot = GeoPlot::new("stage10_bench")
                .height(1080.0)
                .geo_bounds(bounds)
                .basemap(basemap)
                .show_legend(false)
                .show_hover(false);
            if let Some((cs, lbl)) = color_bar {
                plot = plot.color_bar(cs, lbl);
            }
            if let Some(ref bl) = bubble_legend {
                plot = plot.bubble_legend(bl.clone());
            }
            let r = if let Some(cache) = tile_cache.as_deref_mut() {
                plot.show_with_tiles(ui, &mut state, layers, cache)
            } else {
                plot.show(ui, &mut state, layers)
            };
            last_prims = geo_primitives(&r.diagnostics);
            last_visited = r.diagnostics.geo.visited_items;
        });
        let ui_elapsed = start.elapsed();
        let meshes = ctx.tessellate(std::mem::take(&mut out.shapes), out.pixels_per_point);
        let total_elapsed = start.elapsed();
        out_meshes = meshes
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
        last_prims,
        out_meshes,
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
