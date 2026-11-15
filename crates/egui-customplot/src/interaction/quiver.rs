use egui::{IdSalt, Pos2};

use crate::{
    HoveredPoint, PlotPoint, PlotTransform, QuiverArrow, QuiverData, YAxis,
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
    pub(super) fn from_quiver_data(point: &HoveredPoint, data: &QuiverData) -> Self {
        let arrow = data.arrows()[point.point_index];
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
/// Returns `(best_hit, visited_count)` so Quiver and Feather track their domain counters cleanly.
pub(super) fn pick_quiver_data(
    data: &QuiverData,
    transform: PlotTransform,
    position: Pos2,
    radius: f32,
    style: ResolvedSeriesStyle,
) -> (Option<(usize, PlotPoint, f32)>, usize) {
    if !style.stroke.width.is_finite() || style.stroke.is_empty() || !style.pattern.is_valid() {
        return (None, 0);
    }
    let mut visited = 0usize;
    let mut closest = None;
    let mut limit = radius;
    for (index, &arrow) in data.arrows().iter().enumerate() {
        visited += 1;
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
    (closest, visited)
}
