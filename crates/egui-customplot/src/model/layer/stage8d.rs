use super::PlotLayer;
use crate::GraphPlot;

impl<'a> From<GraphPlot<'a>> for PlotLayer<'a> {
    fn from(value: GraphPlot<'a>) -> Self {
        Self::Graph(value)
    }
}
