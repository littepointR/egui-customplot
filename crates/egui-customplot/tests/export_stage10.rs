#![cfg(feature = "export")]

use egui::Color32;
use egui_customplot::{
    ColorScale, ExportOptions, Figure, GeoAttribution, GeoBasemapStyle, GeoBounds, GeoBubbleData,
    GeoBubbles, GeoDensity, GeoDensityData, GeoLayer, GeoLine, GeoOfflinePolicy, GeoPathData,
    GeoPlot, GeoPlotState, GeoRasterTile, GeoScatter, GeoScatterData, GeoTileCache, GeoTileKey,
};

#[path = "support/export.rs"]
mod export_support;

#[test]
fn stage10_geographic_figure_exports_self_contained_svg_and_png_after_host_drop() {
    let route =
        GeoPathData::from_lat_lon(&[35.68, 21.31, 37.77], &[139.69, -157.86, -122.42]).unwrap();
    let bubbles = GeoBubbleData::new(
        &[35.68, 21.31, 37.77, -33.87],
        &[139.69, -157.86, -122.42, 151.21],
        &[14.0, 1.0, 3.8, 5.3],
    )
    .unwrap()
    .with_categories(&["WestPac", "MidPac", "EastPac", "SouthPac"])
    .unwrap();
    let density = GeoDensityData::weighted(
        &[48.86, 51.51, 52.52, 41.9, 40.4],
        &[2.35, -0.13, 13.4, 12.5, -3.7],
        &[1.0, 1.5, 1.2, 0.9, 1.1],
    )
    .unwrap()
    .radius_km(380.0)
    .unwrap();
    let stations =
        GeoScatterData::from_lat_lon(&[48.86, 51.51, 52.52, 41.9], &[2.35, -0.13, 13.4, 12.5])
            .unwrap();

    let mut tile_cache = GeoTileCache::new(
        16 * 1024,
        GeoAttribution::new("Synthetic local tiles").with_license("CC0-1.0"),
    )
    .unwrap()
    .with_offline_policy(GeoOfflinePolicy::FallbackToVector);
    let tile = GeoRasterTile::from_rgba(8, 8, [180, 215, 240, 255].repeat(64)).unwrap();
    tile_cache
        .insert(GeoTileKey::new(1, 0, 0).unwrap(), tile.clone())
        .unwrap();
    tile_cache
        .insert(GeoTileKey::new(1, 1, 0).unwrap(), tile)
        .unwrap();

    let bubble_legend = bubbles.bubble_legend("Metro (M)").steps(3);
    let cs = ColorScale::new([0.0, 4.0]).unwrap();
    let mut st0 = GeoPlotState::default();
    let mut st1 = GeoPlotState::default();

    let snap = export_support::snapshot(|ui| {
        let fig = Figure::new("export_stage10")
            .columns(2)
            .cell_height(280.0)
            .show(ui, 2, |cell| match cell.index() {
                0 => cell
                    .show_geo(
                        GeoPlot::new("pac_geo")
                            .title("Pacific Route & Bubbles")
                            .lat_lim([-45.0, 50.0])
                            .lon_lim([125.0, -110.0])
                            .basemap(GeoBasemapStyle::OfflineVectorLight)
                            .bubble_legend(bubble_legend.clone()),
                        &mut st0,
                        &[
                            GeoLayer::from(GeoLine::new(&route).name("Flight")),
                            GeoLayer::from(GeoBubbles::new(&bubbles).name("Hubs")),
                        ],
                    )
                    .into_plot_response(),
                _ => cell
                    .show_geo_with_tiles(
                        GeoPlot::new("eu_geo")
                            .title("Density & Host Tiles")
                            .geo_bounds(
                                GeoBounds::from_lat_lon([-50.0, 60.0], [-90.0, 90.0]).unwrap(),
                            )
                            .basemap(GeoBasemapStyle::HostTiles)
                            .color_bar(cs, "Density"),
                        &mut st1,
                        &[
                            GeoLayer::from(GeoDensity::new(&density).name("KDE").color_scale(cs)),
                            GeoLayer::from(GeoScatter::new(&stations).name("Stations")),
                        ],
                        &mut tile_cache,
                    )
                    .into_plot_response(),
            });
        fig.unwrap().response.rect
    });

    drop((route, bubbles, density, stations, tile_cache, st0, st1));

    let opts = ExportOptions {
        pixels_per_point: 1.0,
        background: Color32::WHITE,
        ..Default::default()
    };
    let svg_bytes = snap.to_svg(opts).unwrap();
    let svg = String::from_utf8(svg_bytes.clone()).unwrap();
    assert!(roxmltree::Document::parse(&svg).is_ok());
    assert!(svg.contains("<svg"));
    assert!(svg.contains("clipPath"));
    assert!(!svg.contains("<text"));
    assert!(svg.contains("data:image/png;base64,"));

    let resvg_px = export_support::render_svg(&svg_bytes, [480, 280]);
    assert!(
        resvg_px
            .as_chunks::<4>()
            .0
            .iter()
            .any(|px| px[..3] != [255, 255, 255])
    );

    let png = snap.to_png(opts).unwrap();
    let (info, rgba) = export_support::decode_png(&png);
    assert!(info.width > 200 && info.height > 150);
    let non_white = rgba
        .as_chunks::<4>()
        .0
        .iter()
        .filter(|px| px[..3] != [255, 255, 255])
        .count();
    assert!(
        non_white > 1_000,
        "exported PNG must contain rendered map content"
    );
}
