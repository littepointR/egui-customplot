use crate::{
    PlotBounds, PlotDiagnostics, PlotLayer, PlotState, YAxis,
    axis::{SeriesTransforms, grid_cell_at},
};
use egui::{IdSalt, Pos2};

#[derive(Clone, Debug)]
pub struct HoveredCell {
    pub layer_id: IdSalt,
    pub layer_index: usize,
    pub column: usize,
    pub row: usize,
    pub value: f64,
    pub bounds: PlotBounds,
    pub y_axis: YAxis,
}

impl HoveredCell {
    pub fn default_text(&self, name: &str) -> String {
        format!(
            "{name}\n[row={}, col={}]  value={:.8e}",
            self.row, self.column, self.value
        )
    }
}

/// The last visible filled heatmap under the pointer wins. Missing values and
/// invalid color/coordinate domains are unpainted and cannot capture a hover.
pub(crate) fn pick_cell(
    state: &PlotState,
    layers: &[PlotLayer<'_>],
    transforms: SeriesTransforms,
    position: Pos2,
    diagnostics: &mut PlotDiagnostics,
) -> Option<HoveredCell> {
    for (layer_index, layer) in layers.iter().enumerate().rev() {
        let (data, colors) = match *layer {
            PlotLayer::Heatmap(heatmap) => (heatmap.data, heatmap.colors),
            PlotLayer::CategoricalHeatmap(cat) => (cat.data.grid(), cat.colors),
            _ => continue,
        };
        if !state.series_visible(layer.id(layer_index)) {
            continue;
        }
        diagnostics.heatmap_pick_queries += 1;
        let transform = transforms.for_axis(layer.y_axis());
        let Some([column, row]) = grid_cell_at(data, transform, position) else {
            continue;
        };
        let value = data.value(column, row).expect("located cell");
        if colors.color(value).is_none() {
            continue;
        }
        return Some(HoveredCell {
            layer_id: layer.id(layer_index),
            layer_index,
            column,
            row,
            value,
            bounds: data.cell_bounds(column, row).expect("validated edges"),
            y_axis: transforms.effective_axis(layer.y_axis()),
        });
    }
    None
}
