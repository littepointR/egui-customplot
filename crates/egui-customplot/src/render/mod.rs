//! CPU geometry preparation and ordered egui Painter submission.

mod animation;
mod annotation;
mod area;
mod axes;
mod bars;
mod bubble_bivariate;
mod cache;
mod color_bar;
mod confusion;
mod contour;
mod curvilinear;
mod distribution;
mod error_bars;
#[cfg(feature = "export")]
mod export;
mod fast_series;
mod flow;
mod geo;
mod geoglobe;
mod graph;
mod heatmap;
mod image;
mod line;
mod line_query;
mod markers;
mod markers_grid_tree;
mod markers_lod;
mod mesh;
mod modern;
mod multivariate;
mod overlay;
mod patch;
mod pattern;
mod pcolor;
mod pie_clouds;
mod plot3d;
mod polar;
mod polygon;
mod quiver;
mod right_axis;
mod series;
mod shapes;
mod sky;
mod smith;
mod spy;
mod stage14_spc_fin;
mod stage14_stats;
mod stage15;
mod stage8c;
mod stems;
mod triangulation;
#[cfg(feature = "wgpu")]
mod wgpu;

pub(crate) use annotation::{paint_annotations, paint_figure_annotations};
pub(crate) use axes::{AxesStyle, paint_axes};
pub use cache::PlotCache;
pub(crate) use color_bar::paint_color_bar;
#[cfg(feature = "export")]
pub use export::{CaptureLimits, ExportError, ExportOptions, PlotSnapshot};
pub(crate) use geo::paint_geo;
pub(crate) use geoglobe::paint_geoglobe;
pub(crate) use overlay::{
    paint_bar_hover, paint_cell_hover, paint_hover, paint_region_hover, paint_selection_overlays,
};
pub(crate) use plot3d::{ZBufferCache, paint_plot3d};
pub(crate) use polar::paint_polar;
pub(crate) use right_axis::paint_right_axis;
pub(crate) use series::paint_series;
pub(crate) use sky::paint_sky;
pub(crate) use smith::paint_smith;
#[cfg(feature = "wgpu")]
pub use wgpu::{WgpuPlotConfig, register_wgpu, register_wgpu_with_config, set_wgpu_budget};

#[cfg(feature = "export")]
#[cfg_attr(not(feature = "wgpu"), allow(dead_code))]
pub(crate) fn is_capturing_snapshot(ctx: &egui::Context) -> bool {
    export::is_capturing_snapshot(ctx)
}

#[cfg(not(feature = "export"))]
#[cfg_attr(not(feature = "wgpu"), allow(dead_code))]
pub(crate) fn is_capturing_snapshot(_ctx: &egui::Context) -> bool {
    false
}
