use egui::{Pos2, Response, Ui, Vec2};

/// Input retained for replay against the final geometry of an egui frame.
#[derive(Clone, Copy, Debug)]
pub(super) enum ViewGesture {
    Reset,
    Drag(Vec2),
    Scroll { anchor: Pos2, delta: f32 },
}

impl ViewGesture {
    pub(super) fn read(ui: &Ui, response: &Response, allow_scroll: bool) -> Option<Self> {
        if response.double_clicked()
            || (response.has_focus() && ui.input(|i| i.key_pressed(egui::Key::Home)))
        {
            return Some(Self::Reset);
        }
        if response.dragged_by(egui::PointerButton::Primary) && response.drag_delta() != Vec2::ZERO
        {
            return Some(Self::Drag(response.drag_delta()));
        }
        if !allow_scroll {
            return None;
        }
        let anchor = response.hover_pos()?;
        ui.input_mut(|i| {
            (i.modifiers == egui::Modifiers::NONE && i.smooth_scroll_delta.y != 0.0).then(|| {
                Self::Scroll {
                    anchor,
                    delta: std::mem::take(&mut i.smooth_scroll_delta.y),
                }
            })
        })
    }
}
