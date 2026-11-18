use egui::{Align2, Color32, FontId, Mesh, Painter, Pos2, Shape, Stroke, epaint};

use crate::{Axis3D, BoxStyle3D, Transform3D, model::ResolvedPlotStyle};

#[allow(clippy::too_many_arguments)]
pub(super) fn paint_axes_3d(
    painter: &Painter,
    previous: [Option<crate::AxisEdge3D>; 3],
    axes: [&Axis3D; 3],
    transform: Transform3D,
    box_style: BoxStyle3D,
    show_grid: bool,
    title: &str,
    style: &ResolvedPlotStyle<'_>,
) -> [Option<crate::AxisEdge3D>; 3] {
    let rect = transform.rect();
    painter.rect_filled(rect, 0.0, style.background);
    if !title.is_empty() {
        painter.text(
            Pos2::new(rect.center().x, rect.top() - 10.0),
            Align2::CENTER_BOTTOM,
            title,
            FontId::proportional(14.0),
            style.text_color,
        );
    }

    let (_, _, forward) = transform.camera().basis();
    let rev = transform.reversed();
    let back_idx = forward.map(|v| if v >= 0.0 { 1.0 } else { 0.0 });

    if box_style == BoxStyle3D::BackPlanes {
        let wall_c = style
            .background
            .blend(Color32::from_rgba_unmultiplied(128, 128, 136, 18));
        draw_back_wall(painter, transform, 0, back_idx[0], wall_c);
        draw_back_wall(painter, transform, 1, back_idx[1], wall_c);
        draw_back_wall(painter, transform, 2, back_idx[2], wall_c);
    }

    let bounds = transform.bounds();
    let scales = transform.mappings();
    let bmin = bounds.min();
    let bmax = bounds.max();
    let ticks = [
        axes[0].ticks([bmin[0], bmax[0]], rect.width() * 0.45),
        axes[1].ticks([bmin[1], bmax[1]], rect.height() * 0.45),
        axes[2].ticks([bmin[2], bmax[2]], rect.height() * 0.45),
    ];

    // Grid lines across the 3 back walls
    for axis in 0..3 {
        if !show_grid || !axes[axis].show_grid() {
            continue;
        }
        let smin = scales[axis].forward(bmin[axis]).unwrap_or(0.0);
        let smax = scales[axis].forward(bmax[axis]).unwrap_or(1.0);
        let o1 = (axis + 1) % 3;
        let o2 = (axis + 2) % 3;
        for tick in &ticks[axis] {
            if !tick.major {
                continue;
            }
            let Some(st) = scales[axis].forward(tick.value) else {
                continue;
            };
            let mut u = ((st - smin) / (smax - smin)).clamp(0.0, 1.0);
            if rev[axis] {
                u = 1.0 - u;
            }
            // Wall orthogonal to o1 at u[o1] = back_idx[o1], spanning u[o2] in 0..1
            let mut p0 = [u; 3];
            let mut p1 = [u; 3];
            p0[o1] = back_idx[o1];
            p1[o1] = back_idx[o1];
            p0[o2] = 0.0;
            p1[o2] = 1.0;
            line_u(painter, transform, p0, p1, style.grid_stroke);
            // Wall orthogonal to o2 at u[o2] = back_idx[o2], spanning u[o1] in 0..1
            let mut q0 = [u; 3];
            let mut q1 = [u; 3];
            q0[o2] = back_idx[o2];
            q1[o2] = back_idx[o2];
            q0[o1] = 0.0;
            q1[o1] = 1.0;
            line_u(painter, transform, q0, q1, style.grid_stroke);
        }
    }

    // Box outline edges (either back-facing walls or all 12 cube edges)
    let box_edges: [([f64; 3], [f64; 3]); 12] = [
        ([0.0, 0.0, 0.0], [1.0, 0.0, 0.0]),
        ([0.0, 1.0, 0.0], [1.0, 1.0, 0.0]),
        ([0.0, 0.0, 1.0], [1.0, 0.0, 1.0]),
        ([0.0, 1.0, 1.0], [1.0, 1.0, 1.0]),
        ([0.0, 0.0, 0.0], [0.0, 1.0, 0.0]),
        ([1.0, 0.0, 0.0], [1.0, 1.0, 0.0]),
        ([0.0, 0.0, 1.0], [0.0, 1.0, 1.0]),
        ([1.0, 0.0, 1.0], [1.0, 1.0, 1.0]),
        ([0.0, 0.0, 0.0], [0.0, 0.0, 1.0]),
        ([1.0, 0.0, 0.0], [1.0, 0.0, 1.0]),
        ([0.0, 1.0, 0.0], [0.0, 1.0, 1.0]),
        ([1.0, 1.0, 0.0], [1.0, 1.0, 1.0]),
    ];
    for (a, b) in box_edges {
        let on_back =
            (0..3).any(|k| (a[k] - b[k]).abs() < 1e-6 && (a[k] - back_idx[k]).abs() < 1e-6);
        let draw = match box_style {
            BoxStyle3D::FullBox => true,
            BoxStyle3D::BackPlanes => on_back,
            BoxStyle3D::AxesOnly => (0..3).any(|k| {
                (a[k] - b[k]).abs() > 0.5 && a[(k + 1) % 3] == 0.0 && a[(k + 2) % 3] == 0.0
            }),
        };
        if draw {
            line_u(painter, transform, a, b, style.border_stroke);
        }
    }

    std::array::from_fn(|i| {
        let context = crate::AxisLayoutContext3D {
            axis: i,
            transform,
            previous: previous[i],
        };
        let placed =
            crate::layout::axis_labels(painter, context, axes[i], &ticks[i], style.text_color);
        for label in &placed.labels {
            if let Some(custom) = &axes[i].decoration {
                custom.paint_label(painter, label);
            } else {
                label.paint(painter);
            }
        }
        Some(placed.edge)
    })
}

fn line_u(painter: &Painter, transform: Transform3D, u0: [f64; 3], u1: [f64; 3], stroke: Stroke) {
    if let (Some((p0, _)), Some((p1, _))) = (transform.project_unit(u0), transform.project_unit(u1))
    {
        painter.line_segment([p0, p1], stroke);
    }
}

fn draw_back_wall(
    painter: &Painter,
    transform: Transform3D,
    axis: usize,
    val: f64,
    color: Color32,
) {
    let o1 = (axis + 1) % 3;
    let o2 = (axis + 2) % 3;
    let corners = [(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)].map(|(a, b)| {
        let mut u = [val; 3];
        u[o1] = a;
        u[o2] = b;
        transform.project_unit(u).map(|(p, _)| p)
    });
    if let [Some(p0), Some(p1), Some(p2), Some(p3)] = corners {
        let mut mesh = Mesh::default();
        for p in [p0, p1, p2, p3] {
            mesh.vertices.push(epaint::Vertex {
                pos: p,
                uv: epaint::WHITE_UV,
                color,
            });
        }
        mesh.indices.extend_from_slice(&[0, 1, 2, 0, 2, 3]);
        painter.add(Shape::mesh(mesh));
    }
}
