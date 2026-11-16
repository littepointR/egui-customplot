use egui::{Rect, Vec2};

/// Computed screen regions for [`crate::GeoPlot`].
#[derive(Clone, Copy, Debug)]
pub(crate) struct GeoLayout {
    pub outer: Rect,
    pub plot_area: Rect,
    pub legend_rect: Option<Rect>,
    pub bubble_rect: Option<Rect>,
    pub color_bar_rect: Option<Rect>,
}

pub(crate) fn compute_geo_layout(
    outer: Rect,
    has_title: bool,
    legend_rows: usize,
    bubble_size: Option<Vec2>,
    has_color_bar: bool,
) -> GeoLayout {
    let title_pad = if has_title { 28.0 } else { 8.0 };
    let left_pad = 44.0;
    let bottom_pad = 26.0;
    let content = Rect::from_min_max(
        egui::pos2(outer.left() + left_pad, outer.top() + title_pad),
        egui::pos2(outer.right() - 8.0, outer.bottom() - bottom_pad),
    );
    let cb_w = if has_color_bar && content.width() >= 240.0 {
        74.0
    } else {
        0.0
    };
    let color_bar_rect = if cb_w > 0.0 && content.height() >= 72.0 {
        Some(Rect::from_min_max(
            egui::pos2(content.right() - cb_w + 4.0, content.top() + 18.0),
            egui::pos2(content.right() - cb_w + 20.0, content.bottom() - 8.0),
        ))
    } else {
        None
    };
    let main_right = content.right() - cb_w;
    let legend_w = if legend_rows > 0 && (main_right - content.left()) >= 220.0 {
        ((main_right - content.left()) * 0.25).clamp(96.0, 176.0)
    } else {
        0.0
    };
    let plot_area = Rect::from_min_max(
        content.min,
        egui::pos2(
            (main_right - legend_w).max(content.left() + 36.0),
            content.bottom().max(content.top() + 36.0),
        ),
    );
    let legend_rect = if legend_rows > 0 && legend_w > 0.0 {
        let h = (legend_rows as f32 * 24.0 + 18.0).min((content.height() - 8.0).max(28.0));
        Some(Rect::from_min_size(
            egui::pos2(main_right - legend_w + 4.0, content.top() + 4.0),
            egui::vec2((legend_w - 8.0).max(40.0), h),
        ))
    } else {
        None
    };
    let bubble_rect = bubble_size.and_then(|size| {
        if plot_area.width() >= size.x + 12.0 && plot_area.height() >= size.y + 12.0 {
            let x = if legend_w > 0.0 && size.x <= legend_w - 4.0 {
                main_right - legend_w + 4.0
            } else {
                plot_area.right() - size.x - 6.0
            };
            let min = egui::pos2(
                x.max(plot_area.left() + 4.0),
                (content.bottom() - size.y - 4.0).max(content.top() + 4.0),
            );
            Some(Rect::from_min_size(min, size))
        } else {
            None
        }
    });
    GeoLayout {
        outer,
        plot_area,
        legend_rect,
        bubble_rect,
        color_bar_rect,
    }
}
