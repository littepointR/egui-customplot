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
        PlotLayer::ConfusionChart(ch) => {
            super::confusion::paint_confusion_chart(painter, ch, transform, text_color, diagnostics)
        }
        PlotLayer::CurvilinearPcolor(pc) => {
            super::curvilinear::paint_curvilinear_pcolor(painter, pc, transform, diagnostics)
        }
        PlotLayer::ProbabilityPlot(prob) => super::stage14_stats::paint_probability_plot(
            painter,
            prob,
            transform,
            resolved,
            pixels_per_point,
            diagnostics,
        ),
        PlotLayer::EcdfPlot(ecdf) => super::stage14_stats::paint_ecdf_plot(
            painter,
            ecdf,
            transform,
            resolved,
            pixels_per_point,
            diagnostics,
        ),
        PlotLayer::DendrogramPlot(dendro) => super::stage14_stats::paint_dendrogram_plot(
            painter,
            dendro,
            transform,
            resolved,
            diagnostics,
        ),
        PlotLayer::Biplot(bp) => super::stage14_stats::paint_biplot(
            painter,
            bp,
            transform,
            resolved,
            text_color,
            pixels_per_point,
            diagnostics,
        ),
        PlotLayer::ControlChart(chart) => super::stage14_spc_fin::paint_control_chart(
            painter,
            chart,
            transform,
            resolved,
            pixels_per_point,
            diagnostics,
        ),
        PlotLayer::Candlestick(candle) => {
            super::stage14_spc_fin::paint_candlestick(painter, candle, transform, diagnostics)
        }
        PlotLayer::OhlcBars(ohlc) => {
            super::stage14_spc_fin::paint_ohlc_bars(painter, ohlc, transform, diagnostics)
        }
        PlotLayer::Waterfall2D(wf) => super::stage14_spc_fin::paint_waterfall2d(
            painter,
            wf,
            transform,
            text_color,
            diagnostics,
        ),
        PlotLayer::ZPlanePlot(zp) => super::stage15::paint_zplane(
            painter,
            zp,
            transform,
            resolved,
            text_color,
            pixels_per_point,
            diagnostics,
        ),
        PlotLayer::NyquistPlot(nyq) => super::stage15::paint_nyquist(
            painter,
            nyq,
            transform,
            resolved,
            text_color,
            pixels_per_point,
            diagnostics,
        ),
        PlotLayer::NicholsPlot(nich) => super::stage15::paint_nichols(
            painter,
            nich,
            transform,
            resolved,
            text_color,
            pixels_per_point,
            diagnostics,
        ),
        PlotLayer::RootLocus(rl) => super::stage15::paint_root_locus(
            painter,
            rl,
            transform,
            resolved,
            text_color,
            pixels_per_point,
            diagnostics,
        ),
        PlotLayer::CwtPlot(cwt) => super::stage15::paint_cwt(painter, cwt, transform, diagnostics),
        PlotLayer::RadarChart(_)
        | PlotLayer::SankeyChart(_)
        | PlotLayer::TreemapChart(_)
        | PlotLayer::SunburstChart(_)
        | PlotLayer::GanttChart(_) => super::modern::paint_modern_layer(
            painter,
            layer,
            transform,
            resolved,
            text_color,
            diagnostics,
        ),
        _ => {}
    }
}
