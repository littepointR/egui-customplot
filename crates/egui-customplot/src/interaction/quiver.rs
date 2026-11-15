use egui::{IdSalt, Pos2};

use crate::{
    HoveredPoint, PlotDiagnostics, PlotPoint, PlotTransform, Quiver, QuiverArrow, YAxis,
    axis::project_vector_segment, model::ResolvedSeriesStyle,
};

/// Which original data-space anchor won the raw-record distance test.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VectorAnchor {
    Origin,
    Endpoint,
}

#[derive(Clone, Debug)]
pub struct HoveredVector {
    pub layer_id: IdSalt,
    pub layer_index: usize,
    pub arrow_index: usize,
    pub arrow: QuiverArrow,
    pub endpoint: PlotPoint,
    pub anchor: VectorAnchor,
    pub y_axis: YAxis,
    pub screen_distance: f32,
}

impl HoveredVector {
    pub fn default_text(&self, name: &str) -> String {
        format!(
            "{name}\n[{}] origin=({:.8e}, {:.8e})\nvector=({:.8e}, {:.8e})\nend=({:.8e}, {:.8e})  {:?}",
            self.arrow_index,
            self.arrow.origin[0],
            self.arrow.origin[1],
            self.arrow.vector[0],
            self.arrow.vector[1],
            self.endpoint[0],
            self.endpoint[1],
            self.anchor,
        )
    }
    pub(super) fn from_point(point: &HoveredPoint, quiver: Quiver<'_>) -> Self {
        let arrow = quiver.data.arrows()[point.point_index];
        Self {
            layer_id: point.series_id,
            layer_index: point.series_index,
            arrow_index: point.point_index,
            arrow,
            endpoint: arrow.endpoint().expect("picked valid vector"),
            anchor: if point.point == arrow.origin {
                VectorAnchor::Origin
            } else {
                VectorAnchor::Endpoint
            },
            y_axis: point.y_axis,
            screen_distance: point.screen_distance,
        }
    }
}

/// No shaft/wing/intersection picking, no reduced data or hidden point index.
/// Return (raw index, anchor, distance); the shared picker owns layer tie rules.
pub(super) fn pick_quiver(
    quiver: Quiver<'_>,
    transform: PlotTransform,
    position: Pos2,
    radius: f32,
    style: ResolvedSeriesStyle,
    diagnostics: &mut PlotDiagnostics,
) -> Option<(usize, PlotPoint, f32)> {
    if !style.stroke.width.is_finite() || style.stroke.is_empty() || !style.pattern.is_valid() {
        return None;
    }
    let mut closest = None;
    let mut limit = radius;
    for (index, &arrow) in quiver.data.arrows().iter().enumerate() {
        diagnostics.visited_pick_quiver_arrows += 1;
        let Some(end) = arrow.endpoint() else {
            continue;
        };
        if !transform.is_valid_point(arrow.origin) || !transform.is_valid_point(end) {
            continue;
        }
        let Ok(Some(segment)) = project_vector_segment(arrow.origin, end, transform) else {
            continue;
        };
        for (point, visible) in [
            (arrow.origin, segment.origin_visible),
            (end, segment.end_visible),
        ] {
            if !visible {
                continue;
            }
            let distance = transform.position_from_point(point).distance(position);
            if distance <= radius && (closest.is_none() || distance < limit) {
                limit = distance;
                closest = Some((index, point, distance));
            }
        }
    }
    closest
}
