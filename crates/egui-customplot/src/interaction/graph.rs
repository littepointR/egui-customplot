use egui::Pos2;

use crate::{
    PlotDiagnostics, PlotLayer, PlotPoint, PlotTransform,
    axis::{graph_edge_screen_polyline, point_polyline_distance},
};

/// Pick the closest visible node in [`PlotLayer::Graph`] within `radius` (or node visual radius).
pub(super) fn pick_graph_node(
    layer: PlotLayer<'_>,
    transform: PlotTransform,
    position: Pos2,
    radius: f32,
    diagnostics: &mut PlotDiagnostics,
) -> Option<Option<(usize, PlotPoint, f32)>> {
    let PlotLayer::Graph(graph) = layer else {
        return None;
    };
    diagnostics.network.pick_queries += 1;
    let mut best: Option<(usize, PlotPoint, f32)> = None;
    for (idx, &coord) in graph.data.coords().iter().enumerate() {
        diagnostics.network.visited_nodes += 1;
        if !transform.is_valid_point(coord) {
            continue;
        }
        let pos = transform.position_from_point(coord);
        let hit_r = radius.max(graph.resolved_node_radius(idx));
        if !pos.is_finite() || !transform.rect().expand(hit_r).contains(pos) {
            continue;
        }
        let dist = pos.distance(position);
        if dist <= hit_r && best.as_ref().is_none_or(|cur| dist < cur.2) {
            best = Some((idx, coord, dist));
        }
    }
    Some(best)
}

/// Pick the closest visible edge in [`PlotLayer::Graph`] (straight, parallel Bezier, or self-loop).
pub(super) fn pick_graph_edge(
    layer: PlotLayer<'_>,
    transform: PlotTransform,
    position: Pos2,
    diagnostics: &mut PlotDiagnostics,
) -> Option<(usize, f64)> {
    let PlotLayer::Graph(graph) = layer else {
        return None;
    };
    diagnostics.network.pick_queries += 1;
    let centroid = {
        let c = graph.data.centroid();
        if transform.is_valid_point(c) {
            transform.position_from_point(c)
        } else {
            transform.rect().center()
        }
    };
    let coords = graph.data.coords();
    let mut best: Option<(usize, f64, f32)> = None;
    for (edge_idx, &edge) in graph.data.edges().iter().enumerate() {
        diagnostics.network.visited_edges += 1;
        let u = coords[edge.source];
        let v = coords[edge.target];
        if !transform.is_valid_point(u) || !transform.is_valid_point(v) {
            continue;
        }
        let up = transform.position_from_point(u);
        let vp = transform.position_from_point(v);
        let (pts, _) = graph_edge_screen_polyline(
            edge,
            up,
            vp,
            centroid,
            graph.data.is_directed(),
            graph.resolved_node_radius(edge.target),
        );
        let dist = point_polyline_distance(position, &pts);
        if dist <= 6.0 && best.as_ref().is_none_or(|cur| dist < cur.2) {
            best = Some((edge_idx, edge.weight, dist));
        }
    }
    best.map(|(idx, weight, _)| (idx, weight))
}
