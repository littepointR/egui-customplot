use egui::{Color32, Mesh, Painter, Shape};

use crate::{
    PlotDiagnostics, PlotImage, PlotTransform,
    axis::{clipped_data_rect, visible_edges},
};

const CELLS_PER_MESH: usize = 16_384;

pub(super) fn paint_image(
    painter: &Painter,
    image: PlotImage<'_>,
    transform: PlotTransform,
    diagnostics: &mut PlotDiagnostics,
) {
    if !image.opacity.is_finite() || image.opacity <= 0.0 {
        return;
    }
    let opacity = image.opacity.clamp(0.0, 1.0);
    let data = image.data;
    let scales = transform.scales();
    let starts = [
        data.domain_start(0, scales[0]),
        data.domain_start(1, scales[1]),
    ];
    let x = data.x_edges();
    let y = data.y_edges();
    let [columns, rows] = visible_edges([x, y], starts, transform, painter.clip_rect());
    let mut mesh = Mesh::default();
    for row in rows {
        for col in columns.clone() {
            diagnostics.images.visited_cells += 1;
            let [r, g, b, a] = data.pixel(col, row).expect("visible image pixel");
            let alpha = (f32::from(a) * opacity).round().clamp(0.0, 255.0) as u8;
            if alpha == 0 {
                continue;
            }
            let Some(rect) =
                clipped_data_rect([x[col], y[row]], [x[col + 1], y[row + 1]], transform)
            else {
                continue;
            };
            let color = Color32::from_rgba_unmultiplied(r, g, b, alpha);
            mesh.add_colored_rect(rect, color);
            diagnostics.images.output_cells += 1;
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
    diagnostics.images.output_meshes += 1;
    painter.add(Shape::mesh(std::mem::take(mesh)));
}
