use super::PlotLayer;
use crate::{
    BinScatter, BoxChart, BubbleChart, BubbleCloud, CategoricalHeatmap, ConfusionChart,
    CurvilinearPcolor, Histogram2, ParallelPlot, PieChart, Raincloud, SwarmChart, ViolinPlot,
    WordCloud,
    model::{
        Biplot, Candlestick, ControlChart, CwtPlot, DendrogramPlot, EcdfPlot, GanttChart,
        NicholsPlot, NyquistPlot, OhlcBars, ProbabilityPlot, RadarChart, RootLocus, SankeyChart,
        SunburstChart, TreemapChart, Waterfall2D, ZPlanePlot,
    },
};

impl<'a> From<BubbleChart<'a>> for PlotLayer<'a> {
    fn from(value: BubbleChart<'a>) -> Self {
        Self::BubbleChart(value)
    }
}
impl<'a> From<SwarmChart<'a>> for PlotLayer<'a> {
    fn from(value: SwarmChart<'a>) -> Self {
        Self::SwarmChart(value)
    }
}
impl<'a> From<Histogram2<'a>> for PlotLayer<'a> {
    fn from(value: Histogram2<'a>) -> Self {
        Self::Histogram2(value)
    }
}
impl<'a> From<BinScatter<'a>> for PlotLayer<'a> {
    fn from(value: BinScatter<'a>) -> Self {
        Self::BinScatter(value)
    }
}
impl<'a> From<BoxChart<'a>> for PlotLayer<'a> {
    fn from(value: BoxChart<'a>) -> Self {
        Self::BoxChart(value)
    }
}
impl<'a> From<ViolinPlot<'a>> for PlotLayer<'a> {
    fn from(value: ViolinPlot<'a>) -> Self {
        Self::ViolinPlot(value)
    }
}
impl<'a> From<Raincloud<'a>> for PlotLayer<'a> {
    fn from(value: Raincloud<'a>) -> Self {
        Self::Raincloud(value)
    }
}
impl<'a> From<PieChart<'a>> for PlotLayer<'a> {
    fn from(value: PieChart<'a>) -> Self {
        Self::PieChart(value)
    }
}
impl<'a> From<WordCloud<'a>> for PlotLayer<'a> {
    fn from(value: WordCloud<'a>) -> Self {
        Self::WordCloud(value)
    }
}
impl<'a> From<BubbleCloud<'a>> for PlotLayer<'a> {
    fn from(value: BubbleCloud<'a>) -> Self {
        Self::BubbleCloud(value)
    }
}
impl<'a> From<CategoricalHeatmap<'a>> for PlotLayer<'a> {
    fn from(value: CategoricalHeatmap<'a>) -> Self {
        Self::CategoricalHeatmap(value)
    }
}
impl<'a> From<ParallelPlot<'a>> for PlotLayer<'a> {
    fn from(value: ParallelPlot<'a>) -> Self {
        Self::ParallelPlot(value)
    }
}
impl<'a> From<ConfusionChart<'a>> for PlotLayer<'a> {
    fn from(value: ConfusionChart<'a>) -> Self {
        Self::ConfusionChart(value)
    }
}
impl<'a> From<CurvilinearPcolor<'a>> for PlotLayer<'a> {
    fn from(value: CurvilinearPcolor<'a>) -> Self {
        Self::CurvilinearPcolor(value)
    }
}
impl<'a> From<ProbabilityPlot<'a>> for PlotLayer<'a> {
    fn from(value: ProbabilityPlot<'a>) -> Self {
        Self::ProbabilityPlot(value)
    }
}
impl<'a> From<EcdfPlot<'a>> for PlotLayer<'a> {
    fn from(value: EcdfPlot<'a>) -> Self {
        Self::EcdfPlot(value)
    }
}
impl<'a> From<DendrogramPlot<'a>> for PlotLayer<'a> {
    fn from(value: DendrogramPlot<'a>) -> Self {
        Self::DendrogramPlot(value)
    }
}
impl<'a> From<Biplot<'a>> for PlotLayer<'a> {
    fn from(value: Biplot<'a>) -> Self {
        Self::Biplot(value)
    }
}
impl<'a> From<ControlChart<'a>> for PlotLayer<'a> {
    fn from(value: ControlChart<'a>) -> Self {
        Self::ControlChart(value)
    }
}
impl<'a> From<Candlestick<'a>> for PlotLayer<'a> {
    fn from(value: Candlestick<'a>) -> Self {
        Self::Candlestick(value)
    }
}
impl<'a> From<OhlcBars<'a>> for PlotLayer<'a> {
    fn from(value: OhlcBars<'a>) -> Self {
        Self::OhlcBars(value)
    }
}
impl<'a> From<Waterfall2D<'a>> for PlotLayer<'a> {
    fn from(value: Waterfall2D<'a>) -> Self {
        Self::Waterfall2D(value)
    }
}
impl<'a> From<ZPlanePlot<'a>> for PlotLayer<'a> {
    fn from(value: ZPlanePlot<'a>) -> Self {
        Self::ZPlanePlot(value)
    }
}
impl<'a> From<NyquistPlot<'a>> for PlotLayer<'a> {
    fn from(value: NyquistPlot<'a>) -> Self {
        Self::NyquistPlot(value)
    }
}
impl<'a> From<NicholsPlot<'a>> for PlotLayer<'a> {
    fn from(value: NicholsPlot<'a>) -> Self {
        Self::NicholsPlot(value)
    }
}
impl<'a> From<RootLocus<'a>> for PlotLayer<'a> {
    fn from(value: RootLocus<'a>) -> Self {
        Self::RootLocus(value)
    }
}
impl<'a> From<CwtPlot<'a>> for PlotLayer<'a> {
    fn from(value: CwtPlot<'a>) -> Self {
        Self::CwtPlot(value)
    }
}
impl<'a> From<RadarChart<'a>> for PlotLayer<'a> {
    fn from(value: RadarChart<'a>) -> Self {
        Self::RadarChart(value)
    }
}
impl<'a> From<SankeyChart<'a>> for PlotLayer<'a> {
    fn from(value: SankeyChart<'a>) -> Self {
        Self::SankeyChart(value)
    }
}
impl<'a> From<TreemapChart<'a>> for PlotLayer<'a> {
    fn from(value: TreemapChart<'a>) -> Self {
        Self::TreemapChart(value)
    }
}
impl<'a> From<SunburstChart<'a>> for PlotLayer<'a> {
    fn from(value: SunburstChart<'a>) -> Self {
        Self::SunburstChart(value)
    }
}
impl<'a> From<GanttChart<'a>> for PlotLayer<'a> {
    fn from(value: GanttChart<'a>) -> Self {
        Self::GanttChart(value)
    }
}
