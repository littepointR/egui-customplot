use egui::{Align2, Color32, Painter, Rect, Stroke, Ui};

use crate::{
    Annotation, AnnotationCoord, PlotDiagnostics, PlotTransform, model::ResolvedPlotStyle,
};

mod primitives;

pub(crate) fn paint_annotations(
    ui: &Ui,
    annotations: &[Annotation],
    transform: PlotTransform,
    style: &ResolvedPlotStyle<'_>,
    diagnostics: &mut PlotDiagnostics,
) {
    let rect = transform.rect();
    let painter = ui.painter().with_clip_rect(rect);
    let font = egui::TextStyle::Small.resolve(ui.style());
    for annotation in annotations {
        let res = match annotation {
            Annotation::Text {
                point,
                text,
                color,
                anchor,
            } => {
                let color = color.unwrap_or(style.text_color);
                if !transform.is_valid_point(*point) {
                    Err(())
                } else if !inside_bounds(*point, transform)
                    || text.is_empty()
                    || color == Color32::TRANSPARENT
                {
                    Ok(false)
                } else {
                    painter.text(
                        transform.position_from_point(*point),
                        *anchor,
                        text.as_ref(),
                        font.clone(),
                        color,
                    );
                    Ok(true)
                }
            }
            Annotation::XLine {
                value,
                label,
                stroke,
            }
            | Annotation::YLine {
                value,
                label,
                stroke,
            } => {
                let axis = usize::from(matches!(annotation, Annotation::YLine { .. }));
                paint_reference_line(
                    ui,
                    &painter,
                    transform,
                    axis,
                    (*value, label, stroke.unwrap_or(style.border_stroke)),
                )
            }
            Annotation::XRegion { range, fill } | Annotation::YRegion { range, fill } => {
                let axis = usize::from(matches!(annotation, Annotation::YRegion { .. }));
                match region_rect(transform, axis, *range) {
                    Ok(Some(r)) if *fill != Color32::TRANSPARENT => {
                        painter.rect_filled(r, 0.0, *fill);
                        Ok(true)
                    }
                    Ok(_) => Ok(false),
                    Err(()) => Err(()),
                }
            }
            other => primitives::paint_ex027(&painter, rect, Some(transform), other, style, &font),
        };
        match res {
            Ok(true) => diagnostics.output_annotations += 1,
            Ok(false) => {}
            Err(()) => diagnostics.annotation_rejections += 1,
        }
    }
}

/// Paints `AnnotationCoord::Normalized` EX-027 overlays onto an arbitrary Figure rect.
pub(crate) fn paint_figure_annotations(
    ui: &Ui,
    annotations: &[Annotation],
    rect: Rect,
    style: &ResolvedPlotStyle<'_>,
) -> (usize, usize) {
    let painter = ui.painter().with_clip_rect(rect);
    let font = egui::TextStyle::Small.resolve(ui.style());
    let mut output = 0usize;
    let mut rejected = 0usize;
    for annotation in annotations {
        let res = match annotation {
            Annotation::Line {
                coord: AnnotationCoord::Normalized,
                ..
            }
            | Annotation::Arrow {
                coord: AnnotationCoord::Normalized,
                ..
            }
            | Annotation::DoubleArrow {
                coord: AnnotationCoord::Normalized,
                ..
            }
            | Annotation::TextArrow {
                coord: AnnotationCoord::Normalized,
                ..
            }
            | Annotation::TextBox {
                coord: AnnotationCoord::Normalized,
                ..
            }
            | Annotation::Rectangle {
                coord: AnnotationCoord::Normalized,
                ..
            }
            | Annotation::Ellipse {
                coord: AnnotationCoord::Normalized,
                ..
            } => primitives::paint_ex027(&painter, rect, None, annotation, style, &font),
            _ => Err(()),
        };
        match res {
            Ok(true) => output += 1,
            Ok(false) => {}
            Err(()) => rejected += 1,
        }
    }
    (output, rejected)
}

fn inside_bounds(point: [f64; 2], transform: PlotTransform) -> bool {
    (0..2).all(|axis| {
        point[axis] >= transform.bounds().min()[axis]
            && point[axis] <= transform.bounds().max()[axis]
    })
}

fn region_rect(transform: PlotTransform, axis: usize, range: [f64; 2]) -> Result<Option<Rect>, ()> {
    if range.iter().any(|v| v.is_nan()) || range[1] < range[0] {
        return Err(());
    }
    let mut min = transform.bounds().min();
    let mut max = transform.bounds().max();
    min[axis] = min[axis].max(range[0]);
    max[axis] = max[axis].min(range[1]);
    if max[axis] <= min[axis] {
        return Ok(None);
    }
    Ok(Some(Rect::from_two_pos(
        transform.position_from_point(min),
        transform.position_from_point(max),
    )))
}

fn paint_reference_line(
    ui: &Ui,
    painter: &Painter,
    transform: PlotTransform,
    axis: usize,
    line: (f64, &str, Stroke),
) -> Result<bool, ()> {
    let (value, label, stroke) = line;
    if transform.scales()[axis].forward(value).is_none()
        || !stroke.width.is_finite()
        || stroke.width < 0.0
    {
        return Err(());
    }
    if stroke.width == 0.0
        || stroke.color == Color32::TRANSPARENT
        || value < transform.bounds().min()[axis]
        || value > transform.bounds().max()[axis]
    {
        return Ok(false);
    }
    let mut min = transform.bounds().min();
    let mut max = transform.bounds().max();
    min[axis] = value;
    max[axis] = value;
    let a = transform.position_from_point(min);
    let b = transform.position_from_point(max);
    painter.line_segment([a, b], stroke);
    if !label.is_empty() {
        let (pos, anchor) = if axis == 0 {
            (
                egui::pos2(a.x + 3.0, transform.rect().top() + 3.0),
                Align2::LEFT_TOP,
            )
        } else {
            (
                egui::pos2(transform.rect().left() + 3.0, a.y - 3.0),
                Align2::LEFT_BOTTOM,
            )
        };
        painter.text(
            pos,
            anchor,
            label,
            egui::TextStyle::Small.resolve(ui.style()),
            stroke.color,
        );
    }
    Ok(true)
}
