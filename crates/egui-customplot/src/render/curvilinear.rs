use egui::Painter;

use super::mesh::{MeshBatch, paint_ring_border, stroke_valid};
use crate::{CurvilinearPcolor, PcolorShading, PlotDiagnostics, PlotTransform, axis::ClipVertex};

pub(super) fn paint_curvilinear_pcolor(
    painter: &Painter,
    pcolor: CurvilinearPcolor<'_>,
    transform: PlotTransform,
    diagnostics: &mut PlotDiagnostics,
) {
    let data = pcolor.data;
    let draw_edges = stroke_valid(pcolor.edge_stroke);
    let mut batch = MeshBatch::default();

    for row in 0..data.cell_rows() {
        for col in 0..data.cell_cols() {
            diagnostics.images.visited_cells += 1;
            let Some(quad) = data.quad_nodes(row, col) else {
                continue;
            };
            if !quad.iter().all(|&p| transform.is_valid_point(p)) {
                diagnostics.numeric_rejections += 1;
                continue;
            }
            let [n00, n10, n11, n01] = quad.map(|p| transform.normalized(p));
            let min_x = n00[0].min(n10[0]).min(n11[0]).min(n01[0]);
            let max_x = n00[0].max(n10[0]).max(n11[0]).max(n01[0]);
            let min_y = n00[1].min(n10[1]).min(n11[1]).min(n01[1]);
            let max_y = n00[1].max(n10[1]).max(n11[1]).max(n01[1]);
            if max_x < 0.0 || min_x > 1.0 || max_y < 0.0 || min_y > 1.0 {
                continue;
            }

            match pcolor.shading {
                PcolorShading::Flat => {
                    let Some(v) = data.node_value(row, col) else {
                        continue;
                    };
                    let Some(c) = pcolor.colors.color(v) else {
                        continue;
                    };
                    let color = c.gamma_multiply(pcolor.alpha);
                    batch.add_flat_triangle(painter, [n00, n10, n11], color, transform);
                    batch.add_flat_triangle(painter, [n00, n11, n01], color, transform);
                }
                PcolorShading::Interp => {
                    let (Some(z00), Some(z10), Some(z11), Some(z01)) = (
                        data.node_value(row, col),
                        data.node_value(row, col + 1),
                        data.node_value(row + 1, col + 1),
                        data.node_value(row + 1, col),
                    ) else {
                        continue;
                    };
                    let (Some(c00), Some(c10), Some(c11), Some(c01)) = (
                        pcolor.colors.color(z00),
                        pcolor.colors.color(z10),
                        pcolor.colors.color(z11),
                        pcolor.colors.color(z01),
                    ) else {
                        continue;
                    };
                    let v00 = ClipVertex::from_color(n00, c00.gamma_multiply(pcolor.alpha));
                    let v10 = ClipVertex::from_color(n10, c10.gamma_multiply(pcolor.alpha));
                    let v11 = ClipVertex::from_color(n11, c11.gamma_multiply(pcolor.alpha));
                    let v01 = ClipVertex::from_color(n01, c01.gamma_multiply(pcolor.alpha));
                    batch.add_gouraud_triangle(painter, [v00, v10, v11], transform);
                    batch.add_gouraud_triangle(painter, [v00, v11, v01], transform);
                }
            }
            diagnostics.images.output_cells += 1;
            if draw_edges {
                paint_ring_border(
                    painter,
                    &quad,
                    true,
                    pcolor.edge_stroke,
                    transform,
                    diagnostics,
                );
            }
        }
    }

    let stats = batch.finish(painter);
    diagnostics.images.output_meshes += stats.meshes;
    diagnostics.numeric_rejections += stats.numeric_rejections;
}
