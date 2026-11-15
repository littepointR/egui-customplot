use egui::{Color32, Stroke, Ui};

use super::{LinePattern, MarkerShape, MarkerStyle, PlotTheme};

/// Optional values distinguish inheritance from explicit transparent/zero/false.
#[derive(Clone, Copy, Debug, Default)]
pub struct SeriesStyle {
    pub color: Option<Color32>,
    pub line_width: Option<f32>,
    pub line_pattern: Option<LinePattern>,
    pub marker_shape: Option<MarkerShape>,
    pub marker_radius: Option<f32>,
    pub line_markers: Option<bool>,
}

impl SeriesStyle {
    /// Explicit values in self override the corresponding base values.
    pub fn over(self, base: Self) -> Self {
        Self {
            color: self.color.or(base.color),
            line_width: self.line_width.or(base.line_width),
            line_pattern: self.line_pattern.or(base.line_pattern),
            marker_shape: self.marker_shape.or(base.marker_shape),
            marker_radius: self.marker_radius.or(base.marker_radius),
            line_markers: self.line_markers.or(base.line_markers),
        }
    }
}

/// An inheritable plot/axes style layer. Parent groups can share a layer without
/// changing the host egui Style. Series-specific values override these defaults.
/// Non-finite/negative plot stroke widths are omitted and counted as numeric
/// rejections. Zero width and transparent colors are explicit overrides.
#[derive(Clone, Copy, Debug, Default)]
pub struct PlotStyle {
    pub background: Option<Color32>,
    pub text_color: Option<Color32>,
    pub grid_stroke: Option<Stroke>,
    pub minor_grid_stroke: Option<Stroke>,
    pub border_stroke: Option<Stroke>,
    pub crosshair_stroke: Option<Stroke>,
    pub series: SeriesStyle,
}

impl PlotStyle {
    pub fn over(self, base: Self) -> Self {
        Self {
            background: self.background.or(base.background),
            text_color: self.text_color.or(base.text_color),
            grid_stroke: self.grid_stroke.or(base.grid_stroke),
            minor_grid_stroke: self.minor_grid_stroke.or(base.minor_grid_stroke),
            border_stroke: self.border_stroke.or(base.border_stroke),
            crosshair_stroke: self.crosshair_stroke.or(base.crosshair_stroke),
            series: self.series.over(base.series),
        }
    }
}

pub(crate) struct ResolvedPlotStyle<'a> {
    pub background: Color32,
    pub text_color: Color32,
    pub grid_stroke: Stroke,
    pub minor_grid_stroke: Stroke,
    pub border_stroke: Stroke,
    pub crosshair_stroke: Stroke,
    pub series: SeriesStyle,
    pub palette: &'a [Color32],
    pub invalid_strokes: usize,
}

#[derive(Clone, Copy)]
pub(crate) struct ResolvedSeriesStyle {
    pub stroke: Stroke,
    pub pattern: LinePattern,
    pub marker: MarkerStyle,
    pub line_markers: bool,
}

const DEFAULT_PALETTE: [Color32; 3] = [
    Color32::from_rgb(70, 160, 235),
    Color32::from_rgb(235, 145, 65),
    Color32::from_rgb(90, 190, 130),
];

pub(crate) fn resolve_plot_style<'a>(
    ui: &Ui,
    theme: Option<&'a PlotTheme>,
    inherited: PlotStyle,
    local: PlotStyle,
) -> ResolvedPlotStyle<'a> {
    let visuals = ui.visuals();
    let grid = Stroke::new(1.0, visuals.widgets.noninteractive.bg_stroke.color);
    let merged = local
        .over(inherited)
        .over(theme.map_or_else(PlotStyle::default, |t| t.style));
    let mut invalid_strokes = 0;
    let mut stroke = |value: Stroke| {
        if value.width.is_finite() && value.width >= 0.0 {
            value
        } else {
            invalid_strokes += 1;
            Stroke::NONE
        }
    };
    ResolvedPlotStyle {
        background: merged.background.unwrap_or(visuals.extreme_bg_color),
        text_color: merged.text_color.unwrap_or(visuals.text_color()),
        grid_stroke: stroke(merged.grid_stroke.unwrap_or(grid)),
        minor_grid_stroke: stroke(
            merged
                .minor_grid_stroke
                .unwrap_or(Stroke::new(0.5, grid.color.gamma_multiply(0.5))),
        ),
        border_stroke: stroke(
            merged
                .border_stroke
                .unwrap_or(visuals.widgets.noninteractive.bg_stroke),
        ),
        crosshair_stroke: stroke(
            merged
                .crosshair_stroke
                .unwrap_or(Stroke::new(1.0, visuals.weak_text_color())),
        ),
        series: merged.series,
        palette: theme
            .filter(|t| !t.palette.is_empty())
            .map_or(&DEFAULT_PALETTE, |t| &t.palette),
        invalid_strokes,
    }
}

impl ResolvedPlotStyle<'_> {
    pub fn resolve_series(&self, index: usize, local: SeriesStyle) -> ResolvedSeriesStyle {
        let merged = local.over(self.series);
        ResolvedSeriesStyle {
            stroke: Stroke::new(
                merged.line_width.unwrap_or(1.5),
                merged
                    .color
                    .unwrap_or(self.palette[index % self.palette.len()]),
            ),
            pattern: merged.line_pattern.unwrap_or_default(),
            marker: MarkerStyle {
                shape: merged.marker_shape.unwrap_or_default(),
                radius: merged.marker_radius.unwrap_or(3.5),
            },
            line_markers: merged.line_markers.unwrap_or(false),
        }
    }
}
