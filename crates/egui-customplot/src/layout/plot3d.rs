use egui::{Rect, Vec2};

use crate::{Axis3D, Bounds3D, Point3D, ScaleMapping3D};

/// Computed screen regions for a [`crate::Plot3D`] widget.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Plot3DLayout {
    pub outer: Rect,
    pub plot_area: Rect,
    pub legend_rect: Option<Rect>,
    pub bubble_rect: Option<Rect>,
    pub color_bar_rect: Option<Rect>,
}

pub(crate) fn compute_plot3d_layout(
    outer: Rect,
    has_title: bool,
    legend_count: usize,
    bubble_size: Option<Vec2>,
    has_color_bar: bool,
) -> Plot3DLayout {
    let title_pad = if has_title && outer.height() >= 48.0 {
        28.0
    } else {
        8.0
    };
    let side_pad = (outer.width() * 0.2).min(8.0);
    let content = Rect::from_min_max(
        egui::pos2(outer.left() + side_pad, outer.top() + title_pad),
        egui::pos2(outer.right() - side_pad, outer.bottom() - 8.0),
    );
    let cb_w = if has_color_bar && content.width() >= 260.0 {
        76.0
    } else {
        0.0
    };
    let color_bar_rect = if cb_w > 0.0 && content.height() >= 72.0 {
        Some(Rect::from_min_max(
            egui::pos2(content.right() - cb_w + 4.0, content.top() + 20.0),
            egui::pos2(content.right() - cb_w + 20.0, content.bottom() - 8.0),
        ))
    } else {
        None
    };
    let content_main_right = content.right() - cb_w;
    let legend_w = if legend_count > 0 && (content_main_right - content.left()) >= 220.0 {
        ((content_main_right - content.left()) * 0.25).clamp(92.0, 168.0)
    } else {
        0.0
    };
    let plot_area = Rect::from_min_max(
        content.min,
        egui::pos2(content_main_right - legend_w, content.bottom()),
    );
    let legend_rect = if legend_count > 0 && legend_w > 0.0 {
        let h = (legend_count as f32 * 24.0 + 16.0).min((content.height() - 8.0).max(24.0));
        Some(Rect::from_min_size(
            egui::pos2(content_main_right - legend_w + 4.0, content.top() + 4.0),
            egui::vec2((legend_w - 8.0).max(40.0), h),
        ))
    } else {
        None
    };
    let bubble_rect = bubble_size.and_then(|size| {
        if plot_area.width() >= size.x + 12.0 && plot_area.height() >= size.y + 12.0 {
            let min = egui::pos2(
                (content_main_right - size.x - 4.0).max(plot_area.left() + 4.0),
                (content.bottom() - size.y - 4.0).max(plot_area.top() + 4.0),
            );
            Some(Rect::from_min_size(min, size))
        } else {
            None
        }
    });
    Plot3DLayout {
        outer,
        plot_area,
        legend_rect,
        bubble_rect,
        color_bar_rect,
    }
}

pub(crate) fn compute_auto_bounds_3d(
    extent: Option<[Point3D; 2]>,
    axes: [&Axis3D; 3],
    scales: [ScaleMapping3D; 3],
) -> Bounds3D {
    let ranges: [[f64; 2]; 3] = std::array::from_fn(|i| {
        axes[i]
            .manual_bounds()
            .filter(|r| scales[i].valid_range(*r))
            .unwrap_or_else(|| scales[i].padded_range(extent.map(|[a, b]| [a[i], b[i]])))
    });
    Bounds3D::new(ranges.map(|r| r[0]), ranges.map(|r| r[1])).expect("validated axis ranges")
}
