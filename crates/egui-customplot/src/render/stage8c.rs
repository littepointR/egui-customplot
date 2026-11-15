use egui::{Color32, Painter};

use crate::{PlotDiagnostics, PlotLayer, PlotTransform, model::ResolvedSeriesStyle};

pub(super) fn paint_stage8c_layer(
    painter: &Painter,
    layer: PlotLayer<'_>,
    transform: PlotTransform,
    resolved: ResolvedSeriesStyle,
    text_color: Color32,
    pixels_per_point: f32,
    diagnostics: &mut PlotDiagnostics,
) {
    match layer {
        PlotLayer::BubbleChart(bubble) => super::bubble_bivariate::paint_bubble_chart(
            painter,
            bubble,
            transform,
            resolved,
            pixels_per_point,
            diagnostics,
        ),
        PlotLayer::SwarmChart(swarm) => super::bubble_bivariate::paint_swarm_chart(
            painter,
            swarm,
            transform,
            resolved,
            pixels_per_point,
            diagnostics,
        ),
        PlotLayer::Histogram2(hist) => {
            super::bubble_bivariate::paint_histogram2(painter, hist, transform, diagnostics)
        }
        PlotLayer::BinScatter(bins) => {
            super::bubble_bivariate::paint_bin_scatter(painter, bins, transform, diagnostics)
        }
        PlotLayer::BoxChart(boxes) => super::distribution::paint_box_chart(
            painter,
            boxes,
            transform,
            resolved,
            pixels_per_point,
            diagnostics,
        ),
        PlotLayer::ViolinPlot(violin) => {
            super::distribution::paint_violin(painter, violin, transform, resolved, diagnostics)
        }
        PlotLayer::Raincloud(rain) => super::distribution::paint_raincloud(
            painter,
            rain,
            transform,
            resolved,
            pixels_per_point,
            diagnostics,
        ),
        PlotLayer::PieChart(pie) => {
            super::pie_clouds::paint_pie_chart(painter, pie, transform, text_color, diagnostics)
        }
        PlotLayer::WordCloud(cloud) => {
            super::pie_clouds::paint_word_cloud(painter, cloud, transform, resolved, diagnostics)
        }
        PlotLayer::BubbleCloud(cloud) => super::pie_clouds::paint_bubble_cloud(
            painter,
            cloud,
            transform,
            text_color,
            diagnostics,
        ),
        PlotLayer::CategoricalHeatmap(cat) => super::multivariate::paint_categorical_heatmap(
            painter,
            cat,
            transform,
            text_color,
            diagnostics,
        ),
        PlotLayer::ParallelPlot(par) => super::multivariate::paint_parallel_plot(
            painter,
            par,
            transform,
            resolved,
            pixels_per_point,
            diagnostics,
        ),
        _ => {}
    }
}
