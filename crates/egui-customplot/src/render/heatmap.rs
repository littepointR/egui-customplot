use egui::{Mesh, Painter, Shape};

use crate::{
    Heatmap, PlotDiagnostics, PlotTransform,
    axis::{grid_cell_rect, visible_grid},
};

const CELLS_PER_MESH: usize = 16_384;

pub(crate) fn paint_heatmap(
    painter: &Painter,
    heatmap: Heatmap<'_>,
    transform: PlotTransform,
    diagnostics: &mut PlotDiagnostics,
) {
    let data = heatmap.data;
    let [columns, rows] = visible_grid(data, transform, painter.clip_rect());
    let mut mesh = Mesh::default();
    for row in rows {
        for column in columns.clone() {
            diagnostics.visited_heatmap_cells += 1;
            let value = data.value(column, row).expect("visible cell");
            let Some(color) = heatmap.colors.color(value) else {
                continue;
            };
            let bounds = data.cell_bounds(column, row).expect("validated cell edges");
            let Some(rect) = grid_cell_rect(bounds, transform) else {
                continue;
            };
            mesh.add_colored_rect(rect, color);
            diagnostics.output_heatmap_cells += 1;
            if mesh.vertices.len() == CELLS_PER_MESH * 4 {
                submit(painter, &mut mesh, diagnostics);
            }
        }
    }
    submit(painter, &mut mesh, diagnostics);
}

fn submit(painter: &Painter, mesh: &mut Mesh, diagnostics: &mut PlotDiagnostics) {
    if mesh.is_empty() {
        return;
    }
    diagnostics.geometry_capacity_bytes += mesh.vertices.capacity()
        * std::mem::size_of::<egui::epaint::Vertex>()
        + mesh.indices.capacity() * std::mem::size_of::<u32>();
    diagnostics.output_heatmap_meshes += 1;
    painter.add(Shape::mesh(std::mem::take(mesh)));
}
