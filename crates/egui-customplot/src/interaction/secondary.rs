use crate::{PlotBounds, PlotState, PlotTransform};

use super::navigation::NavigationResult;

pub(crate) struct SecondaryNavigation {
    pub transform: PlotTransform,
    pub changed: bool,
    pub invalid_manual_range: bool,
}

/// Replay the already-arbitrated action in the secondary Y coordinate system.
/// X is always copied from the final primary transform, never navigated twice.
pub(crate) fn navigate_secondary(
    state: &mut PlotState,
    auto: PlotTransform,
    primary: &NavigationResult,
) -> SecondaryNavigation {
    let base = state
        .frame
        .as_ref()
        .expect("initialized frame")
        .right_y_base;
    let with_y = |range: [f64; 2]| {
        let bounds = auto.bounds();
        PlotBounds::new([bounds.min()[0], range[0]], [bounds.max()[0], range[1]])
            .ok()
            .and_then(|b| auto.with_bounds(b))
    };
    let mut candidate = base.filter(|range| with_y(*range).is_some());
    let invalid_manual_range = base.is_some() && candidate.is_none();
    let mut transform = candidate.and_then(with_y).unwrap_or(auto);
    if primary.reset {
        candidate = None;
    } else {
        if let Some(delta) = primary.applied_pan
            && delta.y != 0.0
            && let Ok(bounds) = transform.pan(egui::vec2(0.0, delta.y))
        {
            candidate = Some([bounds.min()[1], bounds.max()[1]]);
            transform = with_y(candidate.unwrap()).expect("validated secondary pan");
        }
        if let Some((pos, factors)) = primary.applied_zoom
            && factors[1] != 1.0
            && let Ok(bounds) = transform.zoom_at(pos, [1.0, factors[1]])
        {
            candidate = Some([bounds.min()[1], bounds.max()[1]]);
        }
    }
    state.right_y_range = candidate;
    SecondaryNavigation {
        transform: candidate.and_then(with_y).unwrap_or(auto),
        changed: candidate != base,
        invalid_manual_range,
    }
}
