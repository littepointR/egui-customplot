use egui::{IdSalt, Pos2};

use crate::{
    Bar, BarOrientation, HoveredPoint, PlotDiagnostics, PlotLayer, PlotState, YAxis,
    axis::{SeriesTransforms, clipped_axis_segment, clipped_data_rect, rect_edges, valid_extent},
    model::ResolvedPlotStyle,
};

#[derive(Clone, Debug)]
pub struct HoveredBar {
    pub layer_id: IdSalt,
    pub layer_index: usize,
    pub bar_index: usize,
    pub bar: Bar,
    pub orientation: BarOrientation,
    pub y_axis: YAxis,
}

impl HoveredBar {
    pub fn default_text(&self, name: &str) -> String {
        format!(
            "{name}\n[{}] position={:.8e}  value={:.8e}\nbaseline={:.8e}  width={:.8e}",
            self.bar_index, self.bar.center, self.bar.value, self.bar.baseline, self.bar.width
        )
    }
    pub(crate) fn as_point(&self) -> HoveredPoint {
        let mut point = [self.bar.center, self.bar.value];
        if self.orientation == BarOrientation::Horizontal {
            point.swap(0, 1);
        }
        HoveredPoint {
            y_axis: self.y_axis,
            series_id: self.layer_id,
            series_index: self.layer_index,
            point_index: self.bar_index,
            point,
            screen_distance: 0.0,
        }
    }
}

/// Last submitted bar wins, including overlaps inside a single bar descriptor.
/// Transparent fills do not capture interiors; outlines capture only their edge.
pub(super) fn pick_bar(
    state: &PlotState,
    layers: &[PlotLayer<'_>],
    transforms: SeriesTransforms,
    position: Pos2,
    diagnostics: &mut PlotDiagnostics,
    style: &ResolvedPlotStyle<'_>,
) -> Option<HoveredBar> {
    for (layer_index, layer) in layers.iter().enumerate().rev() {
        let PlotLayer::Bars(bars) = *layer else {
            continue;
        };
        if !state.series_visible(layer.id(layer_index)) {
            continue;
        }
        let fill =
            layer.resolved_style(layer_index, style).stroke.color != egui::Color32::TRANSPARENT;
        let border = bars.border;
        let outline = border.width.is_finite() && border.width > 0.0 && !border.is_empty();
        if !fill && !outline {
            continue;
        }
        let transform = transforms.for_axis(bars.y_axis);
        for (bar_index, &bar) in bars.data.bars().iter().enumerate().rev() {
            diagnostics.visited_pick_bars += 1;
            let Some((lo, hi)) = bar.extent(bars.orientation) else {
                continue;
            };
            if lo[0] == hi[0] || lo[1] == hi[1] || !valid_extent(lo, hi, transform) {
                continue;
            }
            let inside =
                fill && clipped_data_rect(lo, hi, transform).is_some_and(|r| r.contains(position));
            let on_border = outline
                && rect_edges(lo, hi).into_iter().any(|(a, b, axis)| {
                    clipped_axis_segment(a, b, axis, transform).is_some_and(|[a, b]| {
                        let closest = egui::pos2(
                            position.x.clamp(a.x.min(b.x), a.x.max(b.x)),
                            position.y.clamp(a.y.min(b.y), a.y.max(b.y)),
                        );
                        closest.distance(position) <= border.width * 0.5 + 1.0
                    })
                });
            if inside || on_border {
                return Some(HoveredBar {
                    layer_id: layer.id(layer_index),
                    layer_index,
                    bar_index,
                    bar,
                    orientation: bars.orientation,
                    y_axis: transforms.effective_axis(bars.y_axis),
                });
            }
        }
    }
    None
}
