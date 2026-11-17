mod hier_gantt;
mod radial_flow;

use egui::{Color32, Painter};

use crate::{PlotDiagnostics, PlotLayer, PlotTransform, model::ResolvedSeriesStyle};

pub(super) fn paint_modern_layer(
    painter: &Painter,
    layer: PlotLayer<'_>,
    transform: PlotTransform,
    resolved: ResolvedSeriesStyle,
    text_color: Color32,
    diagnostics: &mut PlotDiagnostics,
) {
    match layer {
        PlotLayer::RadarChart(r) => {
            radial_flow::paint_radar_chart(painter, r, transform, resolved, text_color, diagnostics)
        }
        PlotLayer::SankeyChart(s) => {
            radial_flow::paint_sankey_chart(painter, s, transform, text_color, diagnostics)
        }
        PlotLayer::TreemapChart(t) => {
            hier_gantt::paint_treemap_chart(painter, t, transform, text_color, diagnostics)
        }
        PlotLayer::SunburstChart(s) => {
            hier_gantt::paint_sunburst_chart(painter, s, transform, text_color, diagnostics)
        }
        PlotLayer::GanttChart(g) => {
            hier_gantt::paint_gantt_chart(painter, g, transform, text_color, diagnostics)
        }
        _ => {}
    }
}
