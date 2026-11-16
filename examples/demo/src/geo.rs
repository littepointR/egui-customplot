//! Host-owned geographic visualization gallery (`geoplot`, `geoscatter`, `geodensityplot`, `geobubble`).
use eframe::egui::{self, Color32};
use egui_customplot::{
    ColorMap, ColorScale, Figure, GeoAttribution, GeoBasemapStyle, GeoBounds, GeoBubbleData,
    GeoBubbles, GeoDensity, GeoDensityData, GeoLayer, GeoLine, GeoOfflinePolicy, GeoPathData,
    GeoPlot, GeoPlotState, GeoProjection, GeoRasterTile, GeoScatter, GeoScatterData, GeoTileCache,
    GeoTileKey, LinePattern, MarkerShape, PlotResponse,
};

pub(super) struct GeoDemo {
    pub enabled: bool,
    pub equirectangular: bool,
    pub dark_basemap: bool,
    pub use_host_tiles: bool,
    pub scroll_zoom: bool,
    transpac_route: GeoPathData,
    stations: GeoScatterData,
    density: GeoDensityData,
    bubbles: GeoBubbleData,
    tile_cache: GeoTileCache,
    states: [GeoPlotState; 4],
}

impl Default for GeoDemo {
    fn default() -> Self {
        let transpac_route = GeoPathData::from_lat_lon(
            &[35.68, 21.31, 37.77, -33.87, -36.85],
            &[139.69, -157.86, -122.42, 151.21, 174.76],
        )
        .expect("route");

        let stations = GeoScatterData::from_lat_lon(
            &[35.68, 21.31, 37.77, 51.51, -22.91, 1.35, -33.87, 64.15],
            &[
                139.69, -157.86, -122.42, -0.13, -43.17, 103.82, 151.21, -21.94,
            ],
        )
        .expect("stations")
        .with_sizes(&[6.0, 8.0, 7.0, 7.5, 6.5, 8.5, 7.0, 5.5])
        .expect("sizes")
        .with_color_values(&[16.0, 26.0, 15.0, 11.0, 25.0, 28.0, 19.0, 5.0])
        .expect("temps");

        let mut d_lat = Vec::with_capacity(240);
        let mut d_lon = Vec::with_capacity(240);
        let mut d_w = Vec::with_capacity(240);
        for i in 0..240 {
            let t = i as f64 * 0.27;
            let (base_lat, base_lon, w) = match i % 3 {
                0 => (48.8, 7.5, 1.4),
                1 => (38.0, -95.0, 1.1),
                _ => (34.0, 118.0, 1.7),
            };
            d_lat.push(base_lat + (t * 1.7).sin() * 6.0);
            d_lon.push(base_lon + (t * 2.3).cos() * 11.0);
            d_w.push(w);
        }
        let density = GeoDensityData::weighted(&d_lat, &d_lon, &d_w)
            .expect("density")
            .radius_km(480.0)
            .expect("radius")
            .grid_resolution(52, 36);

        let bubbles = GeoBubbleData::new(
            &[35.68, 31.23, 28.61, 51.51, 40.71, -23.55, -33.87, 30.04],
            &[139.69, 121.47, 77.21, -0.13, -74.01, -46.63, 151.21, 31.24],
            &[37.4, 29.2, 32.9, 9.6, 19.6, 22.6, 5.4, 22.1],
        )
        .expect("bubbles")
        .with_categories(&[
            "Asia-Pacific",
            "Asia-Pacific",
            "Asia-Pacific",
            "EMEA",
            "Americas",
            "Americas",
            "Asia-Pacific",
            "EMEA",
        ])
        .expect("categories")
        .with_labels(&[
            "Tokyo",
            "Shanghai",
            "Delhi",
            "London",
            "New York",
            "Sao Paulo",
            "Sydney",
            "Cairo",
        ])
        .expect("labels");

        let mut tile_cache = GeoTileCache::new(
            64 * 1024,
            GeoAttribution::new("Synthetic local raster tile cache").with_license("CC0-1.0"),
        )
        .expect("tile cache")
        .with_offline_policy(GeoOfflinePolicy::FallbackToVector);
        let tile =
            GeoRasterTile::from_rgba(16, 16, [186, 218, 240, 255].repeat(256)).expect("tile");
        tile_cache
            .insert(GeoTileKey::new(1, 0, 0).expect("k00"), tile.clone())
            .expect("insert");
        tile_cache
            .insert(GeoTileKey::new(1, 1, 0).expect("k10"), tile)
            .expect("insert");

        Self {
            enabled: false,
            equirectangular: false,
            dark_basemap: false,
            use_host_tiles: false,
            scroll_zoom: false,
            transpac_route,
            stations,
            density,
            bubbles,
            tile_cache,
            states: std::array::from_fn(|_| GeoPlotState::default()),
        }
    }
}

impl GeoDemo {
    pub fn controls(&mut self, ui: &mut egui::Ui) {
        ui.separator();
        ui.checkbox(&mut self.enabled, "Geographic Plots Gallery (--geo)");
        if !self.enabled {
            return;
        }
        ui.checkbox(&mut self.equirectangular, "Equirectangular projection");
        ui.checkbox(&mut self.dark_basemap, "Dark offline vector basemap");
        ui.checkbox(&mut self.use_host_tiles, "Synthetic HostTiles + fallback");
        ui.checkbox(&mut self.scroll_zoom, "Allow map scroll zoom");
        if ui.button("Reset geo viewports").clicked() {
            for st in &mut self.states {
                st.reset();
            }
        }
    }

    pub fn show(&mut self, ui: &mut egui::Ui) -> Vec<PlotResponse> {
        let proj = if self.equirectangular {
            GeoProjection::Equirectangular
        } else {
            GeoProjection::WebMercator
        };
        let vector_basemap = if self.dark_basemap {
            GeoBasemapStyle::OfflineVectorDark
        } else {
            GeoBasemapStyle::OfflineVectorLight
        };
        let cell_h = ((ui.available_height() - 28.0) * 0.5).max(220.0);
        let scatter_scale = ColorScale::new([0.0, 32.0])
            .expect("scatter scale")
            .color_map(ColorMap::Viridis);
        let density_scale = ColorScale::new([0.0, 6.0])
            .expect("density scale")
            .color_map(ColorMap::BlueRed);
        let bubble_legend = self.bubbles.bubble_legend("Pop (M)").steps(3);
        let [s0, s1, s2, s3] = &mut self.states;

        Figure::new("geo-gallery")
            .columns(2)
            .cell_height(cell_h)
            .show(ui, 4, |cell| match cell.index() {
                0 => cell
                    .show_geo(
                        GeoPlot::new("geo-pacific")
                            .title("geoplot (Pacific dateline-crossing)")
                            .projection(proj)
                            .basemap(vector_basemap)
                            .lat_lim([-45.0, 50.0])
                            .lon_lim([125.0, -110.0])
                            .allow_scroll_zoom(self.scroll_zoom),
                        s0,
                        &[GeoLayer::from(
                            GeoLine::new(&self.transpac_route)
                                .name("Pacific Arc")
                                .color(Color32::from_rgb(225, 85, 55))
                                .width(2.2)
                                .pattern(LinePattern::Dashed {
                                    length: 8.0,
                                    gap: 4.0,
                                }),
                        )],
                    )
                    .into_plot_response(),
                1 => cell
                    .show_geo(
                        GeoPlot::new("geo-scatter")
                            .title("geoscatter (per-point size & color)")
                            .projection(proj)
                            .basemap(vector_basemap)
                            .geo_bounds(GeoBounds::world())
                            .color_bar(scatter_scale, "Temp (°C)")
                            .allow_scroll_zoom(self.scroll_zoom),
                        s1,
                        &[GeoLayer::from(
                            GeoScatter::new(&self.stations)
                                .name("Stations")
                                .marker(MarkerShape::Diamond, 6.0)
                                .color_scale(scatter_scale),
                        )],
                    )
                    .into_plot_response(),
                2 => {
                    let plot = GeoPlot::new("geo-density")
                        .title("geodensityplot (spherical kernel + tiles)")
                        .projection(proj)
                        .basemap(if self.use_host_tiles {
                            GeoBasemapStyle::HostTiles
                        } else {
                            vector_basemap
                        })
                        .geo_bounds(
                            GeoBounds::from_lat_lon([-50.0, 62.0], [-130.0, 145.0])
                                .expect("density bounds"),
                        )
                        .color_bar(density_scale, "Density")
                        .allow_scroll_zoom(self.scroll_zoom);
                    if self.use_host_tiles {
                        cell.show_geo_with_tiles(
                            plot,
                            s2,
                            &[GeoLayer::from(
                                GeoDensity::new(&self.density)
                                    .name("Hotspots")
                                    .color_scale(density_scale),
                            )],
                            &mut self.tile_cache,
                        )
                        .into_plot_response()
                    } else {
                        cell.show_geo(
                            plot,
                            s2,
                            &[GeoLayer::from(
                                GeoDensity::new(&self.density)
                                    .name("Hotspots")
                                    .color_scale(density_scale),
                            )],
                        )
                        .into_plot_response()
                    }
                }
                _ => cell
                    .show_geo(
                        GeoPlot::new("geo-bubble")
                            .title("geobubble (categories & BubbleLegend)")
                            .projection(proj)
                            .basemap(vector_basemap)
                            .geo_bounds(GeoBounds::world())
                            .bubble_legend(bubble_legend.clone())
                            .allow_scroll_zoom(self.scroll_zoom),
                        s3,
                        &[GeoLayer::from(
                            GeoBubbles::new(&self.bubbles).name("Metros"),
                        )],
                    )
                    .into_plot_response(),
            })
            .expect("geo gallery figure")
            .inner
    }
}
