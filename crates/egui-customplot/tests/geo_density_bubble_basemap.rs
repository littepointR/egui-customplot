use egui::{Context, Event, Pos2, RawInput, Rect};
use egui_customplot::{
    GeoAttribution, GeoBasemapStyle, GeoBounds, GeoBubbleData, GeoBubbles, GeoDensity,
    GeoDensityData, GeoError, GeoLayer, GeoLine, GeoOfflinePolicy, GeoPathData, GeoPlot,
    GeoPlotState, GeoPoint, GeoProjection, GeoRasterTile, GeoTileCache, GeoTileKey, GeoTransform,
};

fn canvas_input(time: f64, events: Vec<Event>) -> RawInput {
    RawInput {
        time: Some(time),
        screen_rect: Some(Rect::from_min_size(Pos2::ZERO, egui::vec2(720.0, 480.0))),
        events,
        ..Default::default()
    }
}

#[test]
fn geodensityplot_spatial_kernel_weights_radius_modes_and_transparency() {
    let lats = [35.0, 35.4, 34.7, -25.0];
    let lons = [139.0, 139.5, 138.8, -50.0];
    let base = GeoDensityData::from_lat_lon(&lats, &lons)
        .unwrap()
        .radius_km(220.0)
        .unwrap()
        .grid_resolution(40, 28);
    let weighted = GeoDensityData::weighted(&lats, &lons, &[2.0, 2.0, 2.0, 2.0])
        .unwrap()
        .radius_km(220.0)
        .unwrap()
        .grid_resolution(40, 28);
    let neg_rejected =
        GeoDensityData::weighted(&[35.0, 36.0], &[139.0, 140.0], &[1.0, -3.0]).unwrap();
    assert_eq!(neg_rejected.domain_invalid_points(), 1);

    let region = GeoBounds::from_lat_lon([30.0, 42.0], [130.0, 148.0]).unwrap();
    let tf = GeoTransform::new(
        region,
        GeoProjection::WebMercator,
        Rect::from_min_size(Pos2::ZERO, egui::vec2(500.0, 360.0)),
    )
    .unwrap();
    let g1 = base.evaluate_viewport(tf);
    let g2 = weighted.evaluate_viewport(tf);
    assert!(g1.max_density > 0.0);
    assert!(
        (g2.max_density / g1.max_density - 2.0).abs() < 1e-6,
        "doubling weights should double density peak"
    );
    // Point at (-25, -50) is well outside 220 km support of [30..42]N window.
    assert_eq!(g1.visited_samples, 3);

    // ZoomAdaptivePixels changes geographic support inversely with zoom.
    let adaptive = GeoDensityData::from_lat_lon(&[35.0, 35.1], &[139.0, 139.1])
        .unwrap()
        .radius_pixels(28.0)
        .unwrap();
    let wide_tf = GeoTransform::new(
        GeoBounds::world(),
        GeoProjection::WebMercator,
        Rect::from_min_size(Pos2::ZERO, egui::vec2(500.0, 360.0)),
    )
    .unwrap();
    let narrow_tf = GeoTransform::new(
        GeoBounds::from_lat_lon([34.0, 36.0], [138.0, 140.0]).unwrap(),
        GeoProjection::WebMercator,
        Rect::from_min_size(Pos2::ZERO, egui::vec2(500.0, 360.0)),
    )
    .unwrap();
    assert!(adaptive.evaluate_viewport(narrow_tf).visited_samples <= 2);
    assert!(wide_tf.zoom_level_equivalent() < narrow_tf.zoom_level_equivalent());

    let ctx = Context::default();
    let mut state = GeoPlotState::default();
    let mut output_cells = 0usize;
    ctx.run_ui(canvas_input(0.0, vec![]), |ui| {
        let r = GeoPlot::new("geo_density")
            .height(360.0)
            .geo_bounds(region)
            .show(
                ui,
                &mut state,
                &[GeoLayer::from(
                    GeoDensity::new(&base).name("hotspot").alpha(0.65),
                )],
            );
        output_cells = r.diagnostics.geo.output_density_cells;
    })
    .drop_without_applying_deltas();
    assert!(output_cells > 0);
}

#[test]
fn geobubble_size_category_mapping_bubble_legend_and_hover() {
    let bubbles = GeoBubbleData::new(
        &[35.68, 51.51, -33.87],
        &[139.69, -0.13, 151.21],
        &[10.0, 40.0, 100.0],
    )
    .unwrap()
    .with_categories(&["Asia", "Europe", "Asia"])
    .unwrap()
    .with_labels(&["Tokyo", "London", "Sydney"])
    .unwrap()
    .radius_range(4.0, 20.0)
    .unwrap();

    // Area scaling: pixel radius squared increases linearly with size between [10, 100].
    let r_lo = f64::from(bubbles.pixel_radius(10.0));
    let r_mid = f64::from(bubbles.pixel_radius(40.0));
    let r_hi = f64::from(bubbles.pixel_radius(100.0));
    let lhs = (r_mid * r_mid - r_lo * r_lo) / (r_hi * r_hi - r_lo * r_lo);
    assert!((lhs - (40.0 - 10.0) / (100.0 - 10.0)).abs() < 1e-4);

    let legend = bubbles.bubble_legend("Pop (M)").steps(3);
    let ctx = Context::default();
    let mut state = GeoPlotState::default();
    let mut london_px = Pos2::ZERO;

    // Frame 0: register widget and inspect legend + bubble counts.
    ctx.run_ui(canvas_input(0.0, vec![]), |ui| {
        let r = GeoPlot::new("geo_bubble")
            .height(380.0)
            .geo_bounds(GeoBounds::world())
            .bubble_legend(legend.clone())
            .show(
                ui,
                &mut state,
                &[GeoLayer::from(GeoBubbles::new(&bubbles).name("cities"))],
            );
        assert_eq!(r.diagnostics.geo.output_bubbles, 3);
        assert_eq!(r.legend_entries.len(), 3); // "cities" + "Asia" + "Europe"
        london_px = r.transform.project(GeoPoint::new(51.51, -0.13)).unwrap();
    })
    .drop_without_applying_deltas();

    // Frame 1: hover picks London with category and label.
    let mut hit = None;
    ctx.run_ui(
        canvas_input(0.1, vec![Event::PointerMoved(london_px)]),
        |ui| {
            let r = GeoPlot::new("geo_bubble")
                .height(380.0)
                .geo_bounds(GeoBounds::world())
                .bubble_legend(legend.clone())
                .show(
                    ui,
                    &mut state,
                    &[GeoLayer::from(GeoBubbles::new(&bubbles).name("cities"))],
                );
            hit = r.hovered_point;
        },
    )
    .drop_without_applying_deltas();
    let hit = hit.expect("should hover London bubble");
    assert_eq!(hit.category.as_deref(), Some("Europe"));
    assert_eq!(hit.label.as_deref(), Some("London"));
    assert_eq!(hit.value, Some(40.0));

    // Hiding category "Asia" filters Tokyo & Sydney out, leaving only London.
    state.set_category_visible("Asia", false);
    let mut visible_bubbles = 0usize;
    ctx.run_ui(canvas_input(0.2, vec![]), |ui| {
        let r = GeoPlot::new("geo_bubble")
            .height(380.0)
            .geo_bounds(GeoBounds::world())
            .bubble_legend(legend.clone())
            .show(
                ui,
                &mut state,
                &[GeoLayer::from(GeoBubbles::new(&bubbles).name("cities"))],
            );
        visible_bubbles = r.diagnostics.geo.output_bubbles;
    })
    .drop_without_applying_deltas();
    assert_eq!(visible_bubbles, 1);
}

#[test]
fn basemap_attribution_bounded_tile_cache_and_offline_fallback_contract() {
    // Reject empty attribution strings.
    let err = GeoTileCache::new(4096, GeoAttribution::new("   ")).unwrap_err();
    assert_eq!(err, GeoError::AttributionRequired);

    // Enforce hard single-tile and byte-budget LRU bounds.
    let tile_bytes = 16 * 16 * 4; // 1024 B each
    let mut cache = GeoTileCache::new(
        2048,
        GeoAttribution::new("Synthetic local raster tiles").with_license("CC0-1.0"),
    )
    .unwrap()
    .with_offline_policy(GeoOfflinePolicy::FallbackToVector);

    let oversized = GeoRasterTile::from_rgba(64, 64, vec![200; 64 * 64 * 4]).unwrap();
    assert!(matches!(
        cache.insert(GeoTileKey::new(1, 0, 0).unwrap(), oversized),
        Err(GeoError::CacheBudgetExceeded { .. })
    ));

    let make_tile = |rgb: [u8; 3]| {
        let mut rgba = Vec::with_capacity(tile_bytes);
        for _ in 0..(16 * 16) {
            rgba.extend_from_slice(&[rgb[0], rgb[1], rgb[2], 255]);
        }
        GeoRasterTile::from_rgba(16, 16, rgba).unwrap()
    };

    let k00 = GeoTileKey::new(1, 0, 0).unwrap();
    let k10 = GeoTileKey::new(1, 1, 0).unwrap();
    let k01 = GeoTileKey::new(1, 0, 1).unwrap();
    cache.insert(k00, make_tile([80, 150, 210])).unwrap();
    cache.insert(k10, make_tile([120, 190, 140])).unwrap();
    assert_eq!(cache.retained_bytes(), 2048);
    // Touch k00 so k10 becomes LRU, then insert k01 -> k10 is evicted.
    assert!(cache.get(k00).is_some());
    cache.insert(k01, make_tile([210, 160, 90])).unwrap();
    assert!(cache.retained_bytes() <= cache.budget_bytes());
    assert_eq!(cache.evictions(), 1);
    assert!(cache.get(k10).is_none());

    // Render with partial cache (k00, k01 cached; k10, k11 missing -> offline vector fallback).
    let bounds = GeoBounds::from_lat_lon([-50.0, 60.0], [-90.0, 90.0]).unwrap();
    let path = GeoPathData::from_lat_lon(&[30.0, 50.0], &[-60.0, 40.0]).unwrap();
    let mut state = GeoPlotState::default();
    let ctx = Context::default();
    let mut hits = 0usize;
    let mut misses = 0usize;
    let mut fallbacks = 0usize;
    let mut fallback_polys = 0usize;
    ctx.run_ui(canvas_input(0.0, vec![]), |ui| {
        let r = GeoPlot::new("geo_host_tiles")
            .height(360.0)
            .basemap(GeoBasemapStyle::HostTiles)
            .geo_bounds(bounds)
            .show_with_tiles(
                ui,
                &mut state,
                &[GeoLayer::from(GeoLine::new(&path).name("flight"))],
                &mut cache,
            );
        hits = r.diagnostics.geo.tile_cache_hits;
        misses = r.diagnostics.geo.tile_cache_misses;
        fallbacks = r.diagnostics.geo.offline_fallback_tiles;
        fallback_polys = r.diagnostics.geo.output_basemap_polygons;
    })
    .drop_without_applying_deltas();
    assert!(hits > 0, "cached z=1 tiles must hit");
    assert!(misses > 0, "uncached tiles must record misses");
    assert_eq!(misses, fallbacks);
    assert!(
        fallback_polys > 0,
        "FallbackToVector policy should draw vector land on missed tiles"
    );
}
