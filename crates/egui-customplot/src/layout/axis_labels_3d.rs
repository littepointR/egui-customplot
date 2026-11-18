use crate::{Axis3D, AxisEdge3D, AxisLabel3D, AxisLayoutContext3D, AxisTick, axis::edge_geometry};
use egui::{Align2, Color32, FontId};

pub(crate) struct AxisLabels3D {
    pub edge: AxisEdge3D,
    pub labels: Vec<AxisLabel3D>,
}

pub(crate) fn axis_labels(
    painter: &egui::Painter,
    context: AxisLayoutContext3D,
    axis: &Axis3D,
    ticks: &[AxisTick],
    color: Color32,
) -> AxisLabels3D {
    let t = context.transform;
    let mut chosen = context.previous.unwrap_or_default();
    let score = |edge| {
        edge_geometry(context, edge).map_or(f32::NEG_INFINITY, |(_, normal, distance)| {
            // Prefer lower horizontal axes and a left-hand vertical axis at comparable distances.
            let preference = if context.axis == 2 {
                -normal.x
            } else {
                normal.y
            };
            distance + preference * 0.1 * t.rect().size().min_elem()
        })
    };
    let mut best = score(chosen) + if context.previous.is_some() { 3.0 } else { 0.0 };
    for a in [false, true] {
        for b in [false, true] {
            let edge = AxisEdge3D::new(a, b);
            let candidate = score(edge);
            if candidate > best {
                best = candidate;
                chosen = edge;
            }
        }
    }
    let mut edge = axis.fixed_edge.unwrap_or(chosen);
    if let Some(custom) = &axis.decoration {
        edge = custom.select_edge(&context, edge);
    }
    let mut labels = Vec::new();
    if axis.is_visible()
        && let Some((mid, normal, _)) = edge_geometry(context, edge)
    {
        let i = context.axis;
        let scale = t.mappings()[i];
        let lo = scale.forward(t.bounds().min()[i]).unwrap();
        let hi = scale.forward(t.bounds().max()[i]).unwrap();
        for tick in ticks
            .iter()
            .filter(|tick| tick.major && !tick.label.is_empty())
        {
            let Some(v) = scale.forward(tick.value) else {
                continue;
            };
            let u = ((v - lo) / (hi - lo)).clamp(0., 1.);
            let oriented = if t.reversed()[i] { 1. - u } else { u };
            if let Some((point, _)) = t.project_unit(edge.unit(i, t.reversed(), oriented)) {
                labels.push(AxisLabel3D {
                    value: Some(tick.value),
                    text: tick.label.clone(),
                    position: point + 12. * normal,
                    align: Align2::CENTER_CENTER,
                    font: FontId::proportional(10.),
                    color,
                });
            }
        }
        if !axis.axis_label().is_empty() {
            labels.push(AxisLabel3D {
                value: None,
                text: axis.axis_label().into(),
                position: mid + 26. * normal,
                align: Align2::CENTER_CENTER,
                font: FontId::proportional(12.),
                color,
            });
        }
        // Reserve the title first, then keep only tick labels with disjoint measured bounds.
        let bounds = |label: &AxisLabel3D| {
            let galley =
                painter.layout_no_wrap(label.text.clone(), label.font.clone(), label.color);
            label
                .align
                .anchor_size(label.position, galley.size())
                .expand(1.)
        };
        let mut occupied: Vec<_> = labels
            .iter()
            .filter(|l| l.value.is_none())
            .map(bounds)
            .collect();
        labels.retain(|l| {
            if l.value.is_none() {
                return true;
            }
            let area = bounds(l);
            if occupied.iter().any(|r| area.intersects(*r)) {
                return false;
            }
            occupied.push(area);
            true
        });
        if let Some(custom) = &axis.decoration {
            custom.layout_labels(&context, edge, &mut labels);
        }
    }
    AxisLabels3D { edge, labels }
}
