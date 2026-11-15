use egui::{Mesh, Painter, Pos2, Shape};

use super::mesh::{MeshBatch, stroke_valid};
use crate::{
    Pcolor, PcolorShading, PlotDiagnostics, PlotTransform,
    axis::{ClipVertex, clipped_axis_segment, clipped_data_rect, rect_edges, visible_edges},
};

const CELLS_PER_MESH: usize = 16_384;

pub(super) fn paint_pcolor(
    painter: &Painter,
    pcolor: Pcolor<'_>,
    transform: PlotTransform,
    diagnostics: &mut PlotDiagnostics,
) {
    let data = pcolor.data;
    let scales = transform.scales();
    let starts = [
        data.domain_start(0, scales[0]),
        data.domain_start(1, scales[1]),
    ];
    let [columns, rows] = visible_edges(
        [data.x_nodes(), data.y_nodes()],
        starts,
        transform,
        painter.clip_rect(),
    );
    let draw_edges = stroke_valid(pcolor.edge_stroke);
    let x = data.x_nodes();
    let y = data.y_nodes();

    match pcolor.shading {
        PcolorShading::Flat => {
            let mut mesh = Mesh::default();
            for row in rows {
                for col in columns.clone() {
                    diagnostics.images.visited_cells += 1;
                    let Some(value) = data.node_value(col, row) else {
                        continue;
                    };
                    let Some(color) = pcolor.colors.color(value) else {
                        continue;
                    };
                    let lo = [x[col], y[row]];
                    let hi = [x[col + 1], y[row + 1]];
                    let Some(rect) = clipped_data_rect(lo, hi, transform) else {
                        continue;
                    };
                    mesh.add_colored_rect(rect, color);
                    diagnostics.images.output_cells += 1;
                    if draw_edges {
                        paint_cell_edges(
                            painter,
                            lo,
                            hi,
                            pcolor.edge_stroke,
                            transform,
                            diagnostics,
                        );
                    }
                    if mesh.vertices.len() == CELLS_PER_MESH * 4 {
                        submit_rect_mesh(painter, &mut mesh, diagnostics);
                    }
                }
            }
            submit_rect_mesh(painter, &mut mesh, diagnostics);
        }
        PcolorShading::Interp => {
            let mut batch = MeshBatch::default();
            for row in rows {
                for col in columns.clone() {
                    diagnostics.images.visited_cells += 1;
                    let (Some(z00), Some(z10), Some(z11), Some(z01)) = (
                        data.node_value(col, row),
                        data.node_value(col + 1, row),
                        data.node_value(col + 1, row + 1),
                        data.node_value(col, row + 1),
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
                    let p00 = [x[col], y[row]];
                    let p11 = [x[col + 1], y[row + 1]];
                    if !transform.is_valid_point(p00) || !transform.is_valid_point(p11) {
                        continue;
                    }
                    let v00 = ClipVertex::from_color(transform.normalized(p00), c00);
                    let v10 = ClipVertex::from_color(transform.normalized([p11[0], p00[1]]), c10);
                    let v11 = ClipVertex::from_color(transform.normalized(p11), c11);
                    let v01 = ClipVertex::from_color(transform.normalized([p00[0], p11[1]]), c01);
                    batch.add_gouraud_triangle(painter, [v00, v10, v11], transform);
                    batch.add_gouraud_triangle(painter, [v00, v11, v01], transform);
                    diagnostics.images.output_cells += 1;
                    if draw_edges {
                        paint_cell_edges(
                            painter,
                            p00,
                            p11,
                            pcolor.edge_stroke,
                            transform,
                            diagnostics,
                        );
                    }
                }
            }
            let stats = batch.finish(painter);
            diagnostics.images.output_meshes += stats.meshes;
            diagnostics.geometry_capacity_bytes += stats.bytes;
            diagnostics.numeric_rejections += stats.numeric_rejections;
        }
    }
}

fn paint_cell_edges(
    painter: &Painter,
    lo: [f64; 2],
    hi: [f64; 2],
    stroke: egui::Stroke,
    transform: PlotTransform,
    diagnostics: &mut PlotDiagnostics,
) {
    for (a, b, axis) in rect_edges(lo, hi) {
        if let Some([pa, pb]) = clipped_axis_segment(a, b, axis, transform) {
            diagnostics.geometry_capacity_bytes += 2 * std::mem::size_of::<Pos2>();
            painter.line_segment([pa, pb], stroke);
        }
    }
}

fn submit_rect_mesh(painter: &Painter, mesh: &mut Mesh, diagnostics: &mut PlotDiagnostics) {
    if mesh.is_empty() {
        return;
    }
    diagnostics.geometry_capacity_bytes += mesh.vertices.capacity()
        * std::mem::size_of::<egui::epaint::Vertex>()
        + mesh.indices.capacity() * std::mem::size_of::<u32>();
    diagnostics.images.output_meshes += 1;
    painter.add(Shape::mesh(std::mem::take(mesh)));
}
