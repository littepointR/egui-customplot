use egui::{Pos2, Rect, Vec2};

/// Computed screen regions for a polar plot widget.
#[derive(Clone, Copy, Debug)]
pub(crate) struct PolarLayout {
    pub outer: Rect,
    pub plot_area: Rect,
    pub center: Pos2,
    pub pixel_radius: f32,
    pub legend_rect: Option<Rect>,
    pub bubble_rect: Option<Rect>,
}

pub(crate) fn compute_polar_layout(
    outer: Rect,
    has_title: bool,
    legend_count: usize,
    bubble_size: Option<Vec2>,
) -> PolarLayout {
    let title_pad = if has_title { 28.0 } else { 8.0 };
    let pad_h = 28.0;
    let pad_v = 28.0;
    let content = Rect::from_min_max(
        egui::pos2(outer.left() + 8.0, outer.top() + title_pad),
        egui::pos2(outer.right() - 8.0, outer.bottom() - 8.0),
    );
    let legend_w = if legend_count > 0 && content.width() >= 220.0 {
        (content.width() * 0.28).clamp(96.0, 180.0)
    } else {
        0.0
    };
    let plot_area = Rect::from_min_max(
        content.min,
        egui::pos2(
            (content.right() - legend_w).max(content.left() + 24.0),
            content.bottom(),
        ),
    );
    let diameter =
        ((plot_area.width() - pad_h * 2.0).min(plot_area.height() - pad_v * 2.0)).max(24.0);
    let pixel_radius = diameter * 0.5;
    let center = plot_area.center();

    let legend_rect = if legend_count > 0 && legend_w > 0.0 {
        let h = (legend_count as f32 * 24.0 + 16.0).min((content.height() - 8.0).max(24.0));
        Some(Rect::from_min_size(
            egui::pos2(content.right() - legend_w + 4.0, content.top() + 4.0),
            egui::vec2((legend_w - 8.0).max(40.0), h),
        ))
    } else {
        None
    };

    let bubble_rect = bubble_size.and_then(|size| {
        if plot_area.width() >= size.x + 12.0 && plot_area.height() >= size.y + 12.0 {
            let min = egui::pos2(
                (content.right() - size.x - 4.0).max(plot_area.left() + 4.0),
                (content.bottom() - size.y - 4.0).max(plot_area.top() + 4.0),
            );
            Some(Rect::from_min_size(min, size))
        } else {
            None
        }
    });

    PolarLayout {
        outer,
        plot_area,
        center,
        pixel_radius,
        legend_rect,
        bubble_rect,
    }
}
