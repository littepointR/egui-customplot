use egui::{Rect, StrokeKind, Ui};

use crate::{Axis, AxisTick, PlotTransform, model::ResolvedPlotStyle};

pub(crate) struct AxesStyle<'a> {
    pub axes: &'a [Axis; 2],
    pub title: &'a str,
    pub style: &'a ResolvedPlotStyle<'a>,
    pub show_grid: bool,
}

pub(crate) fn paint_axes(
    ui: &Ui,
    outer: Rect,
    transform: PlotTransform,
    ticks: &[Vec<AxisTick>; 2],
    style: AxesStyle<'_>,
) {
    let painter = ui.painter().with_clip_rect(outer);
    let rect = transform.rect();
    let colors = style.style;
    painter.rect_filled(rect, 0.0, colors.background);
    let grid = colors.grid_stroke;
    let font = egui::TextStyle::Small.resolve(ui.style());
    let min = transform.bounds().min();
    for axis in 0..2 {
        for tick in &ticks[axis] {
            let mut point = min;
            point[axis] = tick.value;
            let p = transform.position_from_point(point);
            if style.show_grid {
                let stroke = if tick.major {
                    grid
                } else {
                    colors.minor_grid_stroke
                };
                let segment = if axis == 0 {
                    [egui::pos2(p.x, rect.top()), egui::pos2(p.x, rect.bottom())]
                } else {
                    [egui::pos2(rect.left(), p.y), egui::pos2(rect.right(), p.y)]
                };
                painter.line_segment(segment, stroke);
            }
            if tick.major {
                let (pos, anchor) = if axis == 0 {
                    (
                        egui::pos2(p.x, rect.bottom() + 4.0),
                        egui::Align2::CENTER_TOP,
                    )
                } else {
                    (
                        egui::pos2(rect.left() - 4.0, p.y),
                        egui::Align2::RIGHT_CENTER,
                    )
                };
                painter.text(pos, anchor, &tick.label, font.clone(), colors.text_color);
            }
        }
    }
    if !style.title.is_empty() {
        painter.text(
            egui::pos2(rect.center().x, outer.top() + 2.0),
            egui::Align2::CENTER_TOP,
            style.title,
            egui::TextStyle::Heading.resolve(ui.style()),
            colors.text_color,
        );
    }
    if !style.axes[0].label.is_empty() {
        painter.text(
            egui::pos2(rect.center().x, outer.bottom() - 2.0),
            egui::Align2::CENTER_BOTTOM,
            &style.axes[0].label,
            font.clone(),
            colors.text_color,
        );
    }
    if !style.axes[1].label.is_empty() {
        let galley = painter.layout_no_wrap(style.axes[1].label.clone(), font, colors.text_color);
        let center = egui::pos2(outer.left() + 10.0, rect.center().y);
        let pos = center - galley.size() * 0.5;
        painter.add(
            egui::epaint::TextShape::new(pos, galley, colors.text_color)
                .with_angle_and_anchor(-std::f32::consts::FRAC_PI_2, egui::Align2::CENTER_CENTER),
        );
    }
    painter.rect_stroke(rect, 0.0, colors.border_stroke, StrokeKind::Inside);
}
