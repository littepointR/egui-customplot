use egui::{Align2, Color32, Painter, Pos2, Shape, Stroke};

use crate::{
    GraphPlot, PlotDiagnostics, PlotTransform, axis::graph_edge_screen_polyline,
    model::ResolvedSeriesStyle,
};

pub(super) fn paint_graph(
    painter: &Painter,
    graph: GraphPlot<'_>,
    transform: PlotTransform,
    style: ResolvedSeriesStyle,
    text_color: Color32,
    diagnostics: &mut PlotDiagnostics,
) {
    let base_edge_color = graph
        .edge_color
        .unwrap_or_else(|| style.stroke.color.gamma_multiply(0.7));
    let base_node_color = graph.node_color.unwrap_or(style.stroke.color);
    let hl = graph.highlight;
    let hl_edges = hl.map_or(&[][..], |h| h.edges);
    let hl_nodes = hl.map_or(&[][..], |h| h.nodes);
    let hl_color = hl.map_or(Color32::from_rgb(220, 45, 45), |h| h.color);

    let centroid = {
        let c = graph.data.centroid();
        if transform.is_valid_point(c) {
            transform.position_from_point(c)
        } else {
            transform.rect().center()
        }
    };
    let coords = graph.data.coords();
    let directed = graph.data.is_directed();
    let edge_labels = graph.data.edge_labels();

    for (edge_idx, &edge) in graph.data.edges().iter().enumerate() {
        diagnostics.network.visited_edges += 1;
        let u = coords[edge.source];
        let v = coords[edge.target];
        if !transform.is_valid_point(u) || !transform.is_valid_point(v) {
            continue;
        }
        let u_pos = transform.position_from_point(u);
        let v_pos = transform.position_from_point(v);
        let (pts, tip) = graph_edge_screen_polyline(
            edge,
            u_pos,
            v_pos,
            centroid,
            directed,
            graph.resolved_node_radius(edge.target),
        );
        if pts.len() < 2 {
            continue;
        }
        let is_hl = hl_edges.contains(&edge_idx);
        let edge_c = if is_hl {
            hl_color
        } else if let Some((scalars, scale)) = graph.edge_scalars {
            scalars
                .get(edge_idx)
                .and_then(|&w| scale.color(w))
                .unwrap_or(base_edge_color)
        } else {
            base_edge_color
        };
        let width = if is_hl {
            graph.edge_width * 1.8
        } else {
            graph.edge_width
        };
        painter.add(Shape::line(pts.clone(), Stroke::new(width, edge_c)));
        diagnostics.network.output_edges += 1;

        if let Some((tip_pos, dir)) = tip {
            let len = graph.arrow_size;
            let hw = len * 0.42;
            let back = Pos2::new(tip_pos.x - dir[0] * len, tip_pos.y - dir[1] * len);
            let p1 = Pos2::new(back.x - dir[1] * hw, back.y + dir[0] * hw);
            let p2 = Pos2::new(back.x + dir[1] * hw, back.y - dir[0] * hw);
            painter.add(Shape::convex_polygon(
                vec![tip_pos, p1, p2],
                edge_c,
                Stroke::NONE,
            ));
            diagnostics.network.output_arrows += 1;
        }

        if graph.show_edge_labels
            && let Some(label) = edge_labels.get(edge_idx)
            && !label.is_empty()
        {
            let mid = pts[pts.len() / 2];
            if transform.rect().contains(mid) {
                painter.text(
                    mid + egui::vec2(0.0, -4.0),
                    Align2::CENTER_BOTTOM,
                    label,
                    egui::FontId::proportional(10.0),
                    text_color,
                );
                diagnostics.network.output_labels += 1;
            }
        }
    }

    let node_labels = graph.data.node_labels();
    let rect = transform.rect().expand(32.0);
    for (node_idx, &coord) in coords.iter().enumerate() {
        diagnostics.network.visited_nodes += 1;
        if !transform.is_valid_point(coord) {
            continue;
        }
        let pos = transform.position_from_point(coord);
        if !pos.is_finite() || !rect.contains(pos) {
            continue;
        }
        let is_hl = hl_nodes.contains(&node_idx);
        let radius = graph.resolved_node_radius(node_idx) * if is_hl { 1.3 } else { 1.0 };
        let fill = if is_hl {
            hl_color
        } else if let Some(colors) = graph.node_colors {
            colors.get(node_idx).copied().unwrap_or(base_node_color)
        } else if let Some((scalars, scale)) = graph.node_scalars {
            scalars
                .get(node_idx)
                .and_then(|&v| scale.color(v))
                .unwrap_or(base_node_color)
        } else {
            base_node_color
        };
        let border = if is_hl {
            Stroke::new(2.0, Color32::WHITE)
        } else {
            Stroke::new(1.0, Color32::from_white_alpha(180))
        };
        painter.circle(pos, radius, fill, border);
        diagnostics.network.output_nodes += 1;

        if graph.show_node_labels
            && let Some(label) = node_labels.get(node_idx)
            && !label.is_empty()
        {
            painter.text(
                pos + egui::vec2(radius + 3.0, 0.0),
                Align2::LEFT_CENTER,
                label,
                egui::FontId::proportional(11.0),
                text_color,
            );
            diagnostics.network.output_labels += 1;
        }
    }
}
