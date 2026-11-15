use super::PlotLayer;
use crate::{
    BinScatter, BoxChart, BubbleChart, BubbleCloud, CategoricalHeatmap, Histogram2, ParallelPlot,
    PieChart, Raincloud, SwarmChart, ViolinPlot, WordCloud,
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
