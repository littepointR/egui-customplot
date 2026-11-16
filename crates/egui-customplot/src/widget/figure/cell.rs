use std::sync::Arc;

use egui::{Rect, Ui};

use crate::{
    Layer3D, Line, Plot, Plot3D, Plot3DResponse, Plot3DState, PlotCache, PlotLayer, PlotResponse,
    PlotState, PlotStyle, PlotTheme, PolarLayer, PolarPlot, PolarPlotResponse, PolarState, Series,
};

pub struct FigureCell<'a> {
    ui: &'a mut Ui,
    index: usize,
    columns: usize,
    rect: Rect,
    style: PlotStyle,
    theme: Option<&'a Arc<PlotTheme>>,
}

impl<'a> FigureCell<'a> {
    pub(super) fn new(
        ui: &'a mut Ui,
        index: usize,
        columns: usize,
        rect: Rect,
        style: PlotStyle,
        theme: Option<&'a Arc<PlotTheme>>,
    ) -> Self {
        Self {
            ui,
            index,
            columns,
            rect,
            style,
            theme,
        }
    }

    pub fn index(&self) -> usize {
        self.index
    }
    pub fn row(&self) -> usize {
        self.index / self.columns
    }
    pub fn column(&self) -> usize {
        self.index % self.columns
    }
    pub fn rect(&self) -> Rect {
        self.rect
    }
    pub fn ui(&mut self) -> &mut Ui {
        self.ui
    }

    /// Inherit figure defaults below the Plot's existing parent/local values,
    /// and size its outer rectangle to the cell. Returns an ordinary Plot.
    pub fn plot(&self, mut plot: Plot) -> Plot {
        plot.inherited_style = plot.inherited_style.over(self.style);
        if plot.theme.is_none() {
            plot.theme = self.theme.cloned();
        }
        plot.height(self.rect.height())
    }

    pub fn show_plot(
        &mut self,
        plot: Plot,
        state: &mut PlotState,
        lines: &[Line<'_>],
    ) -> PlotResponse {
        self.plot(plot).show(self.ui, state, lines)
    }

    pub fn show_series(
        &mut self,
        plot: Plot,
        state: &mut PlotState,
        series: &[Series<'_>],
    ) -> PlotResponse {
        self.plot(plot).show_series(self.ui, state, series)
    }

    pub fn show_series_cached(
        &mut self,
        plot: Plot,
        state: &mut PlotState,
        series: &[Series<'_>],
        cache: &mut PlotCache,
    ) -> PlotResponse {
        self.plot(plot)
            .show_series_cached(self.ui, state, series, cache)
    }

    pub fn show_layers(
        &mut self,
        plot: Plot,
        state: &mut PlotState,
        layers: &[PlotLayer<'_>],
    ) -> PlotResponse {
        self.plot(plot).show_layers(self.ui, state, layers)
    }

    pub fn show_layers_cached(
        &mut self,
        plot: Plot,
        state: &mut PlotState,
        layers: &[PlotLayer<'_>],
        cache: &mut PlotCache,
    ) -> PlotResponse {
        self.plot(plot)
            .show_layers_cached(self.ui, state, layers, cache)
    }

    /// Inherit figure defaults and size the [`PolarPlot`] to this cell.
    pub fn polar_plot(&self, mut plot: PolarPlot) -> PolarPlot {
        plot.inherited_style = plot.inherited_style.over(self.style);
        if plot.theme.is_none() {
            plot.theme = self.theme.cloned();
        }
        plot.height(self.rect.height())
    }

    pub fn show_polar(
        &mut self,
        plot: PolarPlot,
        state: &mut PolarState,
        layers: &[PolarLayer<'_>],
    ) -> PolarPlotResponse {
        self.polar_plot(plot).show(self.ui, state, layers)
    }

    /// Inherit figure defaults and size the [`Plot3D`] to this cell.
    pub fn plot3d(&self, mut plot: Plot3D) -> Plot3D {
        plot.inherited_style = plot.inherited_style.over(self.style);
        if plot.theme.is_none() {
            plot.theme = self.theme.cloned();
        }
        plot.height(self.rect.height())
    }

    pub fn show_plot3d(
        &mut self,
        plot: Plot3D,
        state: &mut Plot3DState,
        layers: &[Layer3D<'_>],
    ) -> Plot3DResponse {
        self.plot3d(plot).show(self.ui, state, layers)
    }
}
