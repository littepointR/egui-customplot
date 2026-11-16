use egui::{Color32, Ui};

use crate::{Axis, Bars, Line, ParetoData, Plot, PlotLayer, PlotResponse, PlotState, YAxis};

/// Dual-Y convenience plot (`plotyy`, `LG-007`) pairing left and right layers cleanly.
pub struct PlotYy<'a> {
    id_salt: &'a str,
    left: Vec<PlotLayer<'a>>,
    right: Vec<PlotLayer<'a>>,
    height: f32,
    title: String,
    x_axis: Axis,
    left_axis: Axis,
    right_axis: Axis,
    show_legend: bool,
}

impl<'a> PlotYy<'a> {
    pub fn new(id_salt: &'a str, left: Vec<PlotLayer<'a>>, right: Vec<PlotLayer<'a>>) -> Self {
        Self {
            id_salt,
            left,
            right,
            height: 320.0,
            title: String::new(),
            x_axis: Axis::new(),
            left_axis: Axis::new(),
            right_axis: Axis::new(),
            show_legend: false,
        }
    }

    pub fn lines(id_salt: &'a str, left: Line<'a>, right: Line<'a>) -> Self {
        Self::new(
            id_salt,
            vec![PlotLayer::from(left.y_axis(YAxis::Left))],
            vec![PlotLayer::from(right.y_axis(YAxis::Right))],
        )
    }

    pub fn height(mut self, height: f32) -> Self {
        if height.is_finite() {
            self.height = height.max(48.0);
        }
        self
    }

    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    pub fn x_axis(mut self, axis: Axis) -> Self {
        self.x_axis = axis;
        self
    }

    pub fn left_axis(mut self, axis: Axis) -> Self {
        self.left_axis = axis;
        self
    }

    pub fn right_axis(mut self, axis: Axis) -> Self {
        self.right_axis = axis;
        self
    }

    pub fn show_legend(mut self, show: bool) -> Self {
        self.show_legend = show;
        self
    }

    pub fn show(self, ui: &mut Ui, state: &mut PlotState) -> PlotResponse {
        let mut layers = Vec::with_capacity(self.left.len() + self.right.len());
        for layer in self.left {
            layers.push(with_y_axis(layer, YAxis::Left));
        }
        for layer in self.right {
            layers.push(with_y_axis(layer, YAxis::Right));
        }
        Plot::new(self.id_salt)
            .height(self.height)
            .title(self.title)
            .x_axis(self.x_axis)
            .y_axis(self.left_axis)
            .right_y_axis(self.right_axis)
            .legend(self.show_legend)
            .show_layers(ui, state, &layers)
    }
}

/// Pareto bar + cumulative percentage dual-axis chart (`pareto`, `ML-038`).
pub struct ParetoChart<'a> {
    id_salt: &'a str,
    data: &'a ParetoData,
    height: f32,
    title: String,
    bar_color: Color32,
    line_color: Color32,
    y_label: String,
}

impl<'a> ParetoChart<'a> {
    pub fn new(id_salt: &'a str, data: &'a ParetoData) -> Self {
        Self {
            id_salt,
            data,
            height: 300.0,
            title: String::new(),
            bar_color: Color32::from_rgb(0, 114, 189),
            line_color: Color32::from_rgb(217, 83, 25),
            y_label: "Value".to_owned(),
        }
    }

    pub fn height(mut self, height: f32) -> Self {
        if height.is_finite() {
            self.height = height.max(48.0);
        }
        self
    }

    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    pub fn y_label(mut self, label: impl Into<String>) -> Self {
        self.y_label = label.into();
        self
    }

    pub fn bar_color(mut self, color: Color32) -> Self {
        self.bar_color = color;
        self
    }

    pub fn line_color(mut self, color: Color32) -> Self {
        self.line_color = color;
        self
    }

    pub fn show(self, ui: &mut Ui, state: &mut PlotState) -> PlotResponse {
        let bars = Bars::new(self.data.bars())
            .name("Count")
            .color(self.bar_color)
            .y_axis(YAxis::Left);
        let line = Line::new(self.data.cumulative_line())
            .name("Cumulative %")
            .color(self.line_color)
            .width(2.0)
            .markers(crate::MarkerStyle::default())
            .y_axis(YAxis::Right);
        let x_axis = Axis::new().categories(self.data.labels().iter().map(|s| s.as_str()));
        let left = Axis::new().label(self.y_label);
        let right = Axis::new()
            .label("Cumulative (%)")
            .formatter(|v| format!("{v:.0}%"));
        Plot::new(self.id_salt)
            .height(self.height)
            .title(self.title)
            .x_axis(x_axis)
            .y_axis(left)
            .right_y_axis(right)
            .legend(true)
            .show_layers(ui, state, &[bars.into(), line.into()])
    }
}

fn with_y_axis(layer: PlotLayer<'_>, axis: YAxis) -> PlotLayer<'_> {
    match layer {
        PlotLayer::Series(crate::Series::Line(s)) => PlotLayer::from(s.y_axis(axis)),
        PlotLayer::Series(crate::Series::Scatter(s)) => PlotLayer::from(s.y_axis(axis)),
        PlotLayer::StreamLine(s) => PlotLayer::from(s.y_axis(axis)),
        PlotLayer::WaveformLine(s) => PlotLayer::from(s.y_axis(axis)),
        PlotLayer::Heatmap(s) => PlotLayer::from(s.y_axis(axis)),
        PlotLayer::Bars(s) => PlotLayer::from(s.y_axis(axis)),
        PlotLayer::ErrorBars(s) => PlotLayer::from(s.y_axis(axis)),
        PlotLayer::Stems(s) => PlotLayer::from(s.y_axis(axis)),
        PlotLayer::Quiver(s) => PlotLayer::from(s.y_axis(axis)),
        PlotLayer::Area(s) => PlotLayer::from(s.y_axis(axis)),
        PlotLayer::Fill(s) => PlotLayer::from(s.y_axis(axis)),
        PlotLayer::Patch(s) => PlotLayer::from(s.y_axis(axis)),
        PlotLayer::Rectangles(s) => PlotLayer::from(s.y_axis(axis)),
        PlotLayer::PolyShape(s) => PlotLayer::from(s.y_axis(axis)),
        PlotLayer::Contours(s) => PlotLayer::from(s.y_axis(axis)),
        PlotLayer::FilledContours(s) => PlotLayer::from(s.y_axis(axis)),
        PlotLayer::Pcolor(s) => PlotLayer::from(s.y_axis(axis)),
        PlotLayer::Image(s) => PlotLayer::from(s.y_axis(axis)),
        PlotLayer::TriPlot(s) => PlotLayer::from(s.y_axis(axis)),
        PlotLayer::Voronoi(s) => PlotLayer::from(s.y_axis(axis)),
        PlotLayer::AlphaShape(s) => PlotLayer::from(s.y_axis(axis)),
        PlotLayer::Streamlines(s) => PlotLayer::from(s.y_axis(axis)),
        PlotLayer::Feather(s) => PlotLayer::from(s.y_axis(axis)),
        PlotLayer::Spy(s) => PlotLayer::from(s.y_axis(axis)),
        PlotLayer::Comet(s) => PlotLayer::from(s.y_axis(axis)),
        PlotLayer::BubbleChart(s) => PlotLayer::from(s.y_axis(axis)),
        PlotLayer::SwarmChart(s) => PlotLayer::from(s.y_axis(axis)),
        PlotLayer::Histogram2(s) => PlotLayer::from(s.y_axis(axis)),
        PlotLayer::BinScatter(s) => PlotLayer::from(s.y_axis(axis)),
        PlotLayer::BoxChart(s) => PlotLayer::from(s.y_axis(axis)),
        PlotLayer::ViolinPlot(s) => PlotLayer::from(s.y_axis(axis)),
        PlotLayer::Raincloud(s) => PlotLayer::from(s.y_axis(axis)),
        PlotLayer::PieChart(s) => PlotLayer::from(s.y_axis(axis)),
        PlotLayer::WordCloud(s) => PlotLayer::from(s.y_axis(axis)),
        PlotLayer::BubbleCloud(s) => PlotLayer::from(s.y_axis(axis)),
        PlotLayer::CategoricalHeatmap(s) => PlotLayer::from(s.y_axis(axis)),
        PlotLayer::ParallelPlot(s) => PlotLayer::from(s.y_axis(axis)),
        PlotLayer::Graph(s) => PlotLayer::from(s.y_axis(axis)),
    }
}
