use egui::{Color32, Rect, Response, Ui, Vec2};

/// Host-provided legend content for 2D and 3D plots. Sizes are logical points.
/// The plot owns clipping, scrolling, identity and once-per-frame visibility changes.
pub trait LegendComponent: Send + Sync {
    /// Preferred size of the whole legend including its frame.
    fn measure(&self, ui: &Ui, names: &[&str]) -> Vec2;
    /// Draw one entry and return the interaction response used for visibility toggles.
    fn item(&self, ui: &mut Ui, name: &str, color: Color32, visible: bool) -> Response;
}

pub(crate) fn reserve_component<'a>(
    ui: &Ui,
    outer: &mut Rect,
    names: impl Iterator<Item = &'a str>,
    component: &dyn LegendComponent,
    position: crate::LegendPosition,
) -> Option<Rect> {
    let names: Vec<_> = names.filter(|name| !name.is_empty()).collect();
    if names.is_empty() || outer.width() < 96. || outer.height() < 60. {
        return None;
    }
    let wanted = component.measure(ui, &names);
    if !wanted.is_finite() || wanted.min_elem() <= 0. {
        return None;
    }
    let size = wanted.min(outer.size());
    let mut rect = *outer;
    match position {
        crate::LegendPosition::OutsideRight => {
            rect.min.x = rect.right() - size.x.min(rect.width() * 0.42);
            rect.max.y = rect.top() + size.y;
            outer.max.x = rect.left() - 4.;
        }
        crate::LegendPosition::OutsideBottom => {
            rect.min.y = rect.bottom() - size.y.min(rect.height() * 0.4);
            rect.max.x = rect.left() + size.x;
            outer.max.y = rect.top() - 4.;
        }
        crate::LegendPosition::InsideTopRight => {
            rect = Rect::from_min_size(rect.right_top() - egui::vec2(size.x, 0.), size);
        }
    }
    Some(rect)
}
