use egui::{IdSalt, Pos2, Rect};

use crate::{
    PlotDiagnostics, PlotLayer, PlotPoint, PlotState, YAxis,
    axis::{
        SeriesTransforms, clipped_data_rect, edge_cell_at, point_in_ring_transformed, valid_extent,
    },
    model::ResolvedPlotStyle,
};

mod hit;
mod stage8c;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RegionKind {
    Area,
    Fill,
    Patch,
    Rectangle,
    PolyShape,
    FilledContour,
    PcolorCell,
    ImagePixel,
    AlphaShape,
    Histogram2Bin,
    BinScatterCell,
    BoxChart,
    Violin,
    Raincloud,
    PieSlice,
    WordCloudItem,
    BubbleCloudItem,
    GraphEdge,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HoveredRegion {
    pub layer_id: IdSalt,
    pub layer_index: usize,
    pub y_axis: YAxis,
    pub kind: RegionKind,
    pub item_index: usize,
    pub grid_index: Option<[usize; 2]>,
    pub scalar_value: Option<f64>,
    pub contour_band: Option<[f64; 2]>,
    pub rgba: Option<[u8; 4]>,
    pub point: PlotPoint,
    pub screen_rect: Option<Rect>,
}

impl HoveredRegion {
    pub fn default_text(&self, name: &str) -> String {
        let mut out = if name.is_empty() {
            format!("x={:.6e}\ny={:.6e}", self.point[0], self.point[1])
        } else {
            format!("{name}\nx={:.6e}\ny={:.6e}", self.point[0], self.point[1])
        };
        if let Some([col, row]) = self.grid_index {
            out.push_str(&format!("\n[row={row}, col={col}]"));
        } else {
            out.push_str(&format!("\n#{}", self.item_index));
        }
        if let Some([lo, hi]) = self.contour_band {
            if hi.is_finite() {
                out.push_str(&format!("  band=[{lo:.6e}, {hi:.6e}]"));
            } else {
                out.push_str(&format!("  band=[{lo:.6e}, +inf)"));
            }
        } else if let Some(val) = self.scalar_value {
            out.push_str(&format!("  value={val:.6e}"));
        }
        if let Some([r, g, b, a]) = self.rgba {
            out.push_str(&format!("  rgba=({r}, {g}, {b}, {a})"));
        }
        out
    }
}

/// Topmost visible filled/raster region under `position` wins. Transparent items,
/// PolyShape holes, invalid Patch/Pcolor values, and transparent image pixels are skipped.
pub(crate) fn pick_region(
    state: &PlotState,
    layers: &[PlotLayer<'_>],
    transforms: SeriesTransforms,
    position: Pos2,
    diagnostics: &mut PlotDiagnostics,
    style: &ResolvedPlotStyle<'_>,
) -> Option<HoveredRegion> {
    for (layer_index, layer) in layers.iter().enumerate().rev() {
        let layer_id = layer.id(layer_index);
        if !state.series_visible(layer_id) {
            continue;
        }
        let transform = transforms.for_axis(layer.y_axis());
        if !transform.rect().contains(position) {
            continue;
        }
        let point = transform.point_from_position(position);
        let p_norm = transform.normalized(point);
        if !p_norm[0].is_finite() || !p_norm[1].is_finite() {
            continue;
        }
        let y_axis = transforms.effective_axis(layer.y_axis());
        let resolved = layer.resolved_style(layer_index, style);
        let hit = match *layer {
            PlotLayer::Area(area) => {
                diagnostics.patches.pick_queries += 1;
                hit::pick_area(area, point, p_norm, transform, resolved.stroke.color).map(|idx| {
                    base_region(layer_id, layer_index, y_axis, RegionKind::Area, idx, point)
                })
            }
            PlotLayer::Fill(fill) => {
                diagnostics.patches.pick_queries += 1;
                if resolved.stroke.color == egui::Color32::TRANSPARENT {
                    None
                } else {
                    (0..fill.data.ring_count())
                        .rev()
                        .find(|&i| {
                            fill.data.ring_triangles(i, transform.scales()).is_some()
                                && point_in_ring_transformed(point, fill.data.ring(i), transform)
                        })
                        .map(|idx| {
                            base_region(layer_id, layer_index, y_axis, RegionKind::Fill, idx, point)
                        })
                }
            }
            PlotLayer::Patch(patch) => {
                diagnostics.patches.pick_queries += 1;
                hit::pick_patch(patch, p_norm, transform, resolved.stroke.color).map(
                    |(idx, scalar_value)| HoveredRegion {
                        scalar_value,
                        ..base_region(layer_id, layer_index, y_axis, RegionKind::Patch, idx, point)
                    },
                )
            }
            PlotLayer::Rectangles(rects) => {
                diagnostics.patches.pick_queries += 1;
                if resolved.stroke.color == egui::Color32::TRANSPARENT {
                    None
                } else {
                    rects
                        .data
                        .rects()
                        .iter()
                        .enumerate()
                        .rev()
                        .find_map(|(idx, rect)| {
                            let (lo, hi) = rect.extent()?;
                            (valid_extent(lo, hi, transform) && rect.contains_data(point)).then(
                                || HoveredRegion {
                                    screen_rect: clipped_data_rect(lo, hi, transform),
                                    ..base_region(
                                        layer_id,
                                        layer_index,
                                        y_axis,
                                        RegionKind::Rectangle,
                                        idx,
                                        point,
                                    )
                                },
                            )
                        })
                }
            }
            PlotLayer::PolyShape(poly) => {
                diagnostics.patches.pick_queries += 1;
                if resolved.stroke.color == egui::Color32::TRANSPARENT {
                    None
                } else {
                    (0..poly.data.region_count())
                        .rev()
                        .find(|&i| {
                            poly.data.region_triangles(i, transform.scales()).is_some()
                                && point_in_ring_transformed(
                                    point,
                                    poly.data.region_outer(i),
                                    transform,
                                )
                                && !(0..poly.data.region_hole_count(i)).any(|h| {
                                    point_in_ring_transformed(
                                        point,
                                        poly.data.region_hole(i, h),
                                        transform,
                                    )
                                })
                        })
                        .map(|idx| {
                            base_region(
                                layer_id,
                                layer_index,
                                y_axis,
                                RegionKind::PolyShape,
                                idx,
                                point,
                            )
                        })
                }
            }
            PlotLayer::FilledContours(fc) => {
                diagnostics.contours.pick_queries += 1;
                hit::pick_filled_contour(fc, p_norm, transform).map(|(idx, lo, hi)| HoveredRegion {
                    scalar_value: Some(lo),
                    contour_band: Some([lo, hi]),
                    ..base_region(
                        layer_id,
                        layer_index,
                        y_axis,
                        RegionKind::FilledContour,
                        idx,
                        point,
                    )
                })
            }
            PlotLayer::Pcolor(pc) => {
                diagnostics.images.pick_queries += 1;
                let scales = transform.scales();
                let starts = [
                    pc.data.domain_start(0, scales[0]),
                    pc.data.domain_start(1, scales[1]),
                ];
                edge_cell_at(
                    [pc.data.x_nodes(), pc.data.y_nodes()],
                    starts,
                    transform,
                    position,
                )
                .and_then(|[col, row]| {
                    let scalar = hit::pcolor_cell_scalar(pc, col, row, p_norm, transform)?;
                    let bounds = [
                        [pc.data.x_nodes()[col], pc.data.y_nodes()[row]],
                        [pc.data.x_nodes()[col + 1], pc.data.y_nodes()[row + 1]],
                    ];
                    Some(HoveredRegion {
                        item_index: row * pc.data.cell_cols() + col,
                        grid_index: Some([col, row]),
                        scalar_value: Some(scalar),
                        screen_rect: clipped_data_rect(bounds[0], bounds[1], transform),
                        ..base_region(
                            layer_id,
                            layer_index,
                            y_axis,
                            RegionKind::PcolorCell,
                            0,
                            point,
                        )
                    })
                })
            }
            PlotLayer::Image(img) if img.opacity.is_finite() && img.opacity > 0.0 => {
                diagnostics.images.pick_queries += 1;
                let scales = transform.scales();
                let starts = [
                    img.data.domain_start(0, scales[0]),
                    img.data.domain_start(1, scales[1]),
                ];
                edge_cell_at(
                    [img.data.x_edges(), img.data.y_edges()],
                    starts,
                    transform,
                    position,
                )
                .and_then(|[col, row]| {
                    let px = img.data.pixel(col, row)?;
                    if px[3] == 0 {
                        return None;
                    }
                    let bounds = [
                        [img.data.x_edges()[col], img.data.y_edges()[row]],
                        [img.data.x_edges()[col + 1], img.data.y_edges()[row + 1]],
                    ];
                    Some(HoveredRegion {
                        item_index: row * img.data.width() + col,
                        grid_index: Some([col, row]),
                        rgba: Some(px),
                        screen_rect: clipped_data_rect(bounds[0], bounds[1], transform),
                        ..base_region(
                            layer_id,
                            layer_index,
                            y_axis,
                            RegionKind::ImagePixel,
                            0,
                            point,
                        )
                    })
                })
            }
            PlotLayer::AlphaShape(alpha) => {
                diagnostics.geometry.pick_queries += 1;
                let transient = crate::Patch::new(alpha.data.patch())
                    .color(resolved.stroke.color)
                    .border(alpha.border);
                hit::pick_patch(transient, p_norm, transform, resolved.stroke.color).map(
                    |(idx, _)| {
                        base_region(
                            layer_id,
                            layer_index,
                            y_axis,
                            RegionKind::AlphaShape,
                            idx,
                            point,
                        )
                    },
                )
            }
            PlotLayer::Graph(_) => {
                crate::interaction::graph::pick_graph_edge(*layer, transform, position, diagnostics)
                    .map(|(edge_idx, weight)| {
                        let mut hit = base_region(
                            layer_id,
                            layer_index,
                            y_axis,
                            RegionKind::GraphEdge,
                            edge_idx,
                            point,
                        );
                        hit.scalar_value = Some(weight);
                        hit
                    })
            }
            _ => stage8c::pick_stage8c_region(
                *layer,
                layer_id,
                layer_index,
                transforms.effective_axis(layer.y_axis()),
                point,
                p_norm,
                transform,
            ),
        };
        if hit.is_some() {
            return hit;
        }
    }
    None
}

fn base_region(
    layer_id: IdSalt,
    layer_index: usize,
    y_axis: YAxis,
    kind: RegionKind,
    item_index: usize,
    point: PlotPoint,
) -> HoveredRegion {
    HoveredRegion {
        layer_id,
        layer_index,
        y_axis,
        kind,
        item_index,
        grid_index: None,
        scalar_value: None,
        contour_band: None,
        rgba: None,
        point,
        screen_rect: None,
    }
}
