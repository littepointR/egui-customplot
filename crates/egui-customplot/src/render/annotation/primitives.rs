use std::f32::consts::TAU;

use egui::{Align2, Color32, FontId, Painter, Pos2, Rect, Shape, Stroke, StrokeKind, epaint};

use crate::{Annotation, AnnotationCoord, PlotPoint, PlotTransform, model::ResolvedPlotStyle};

pub(super) fn paint_ex027(
    painter: &Painter,
    rect: Rect,
    transform: Option<PlotTransform>,
    annotation: &Annotation,
    style: &ResolvedPlotStyle<'_>,
    font: &FontId,
) -> Result<bool, ()> {
    match annotation {
        Annotation::Line {
            coord,
            start,
            end,
            stroke,
        } => {
            let stroke = valid_stroke(stroke.unwrap_or(style.border_stroke))?;
            if stroke.is_empty() {
                return Ok(false);
            }
            let (pa, pb) = resolve_pair(*coord, *start, *end, rect, transform)?;
            painter.line_segment([pa, pb], stroke);
            Ok(true)
        }
        Annotation::Arrow {
            coord,
            start,
            end,
            stroke,
            head_size,
        } => {
            let stroke = valid_stroke(stroke.unwrap_or(style.border_stroke))?;
            if !valid_head(*head_size) {
                return Err(());
            }
            if stroke.is_empty() {
                return Ok(false);
            }
            let (pa, pb) = resolve_pair(*coord, *start, *end, rect, transform)?;
            painter.line_segment([pa, pb], stroke);
            paint_arrowhead(painter, pa, pb, *head_size, stroke);
            Ok(true)
        }
        Annotation::DoubleArrow {
            coord,
            start,
            end,
            stroke,
            head_size,
        } => {
            let stroke = valid_stroke(stroke.unwrap_or(style.border_stroke))?;
            if !valid_head(*head_size) {
                return Err(());
            }
            if stroke.is_empty() {
                return Ok(false);
            }
            let (pa, pb) = resolve_pair(*coord, *start, *end, rect, transform)?;
            painter.line_segment([pa, pb], stroke);
            paint_arrowhead(painter, pa, pb, *head_size, stroke);
            paint_arrowhead(painter, pb, pa, *head_size, stroke);
            Ok(true)
        }
        Annotation::TextArrow {
            coord,
            tail,
            tip,
            text,
            stroke,
            text_color,
            head_size,
        } => {
            let stroke = valid_stroke(stroke.unwrap_or(style.border_stroke))?;
            if !valid_head(*head_size) {
                return Err(());
            }
            let (pa, pb) = resolve_pair(*coord, *tail, *tip, rect, transform)?;
            if !stroke.is_empty() {
                painter.line_segment([pa, pb], stroke);
                paint_arrowhead(painter, pa, pb, *head_size, stroke);
            }
            let tcol = text_color.unwrap_or(style.text_color);
            if !text.is_empty() && tcol != Color32::TRANSPARENT {
                let anchor = tail_anchor(pa, pb);
                painter.text(pa, anchor, text.as_ref(), font.clone(), tcol);
            }
            Ok(!stroke.is_empty() || (!text.is_empty() && tcol != Color32::TRANSPARENT))
        }
        Annotation::TextBox {
            coord,
            bounds,
            text,
            fill,
            stroke,
            text_color,
        } => {
            let stroke = valid_stroke(stroke.unwrap_or(style.border_stroke))?;
            let box_rect = resolve_box(*coord, *bounds, rect, transform)?;
            if *fill != Color32::TRANSPARENT {
                painter.rect_filled(box_rect, 0.0, *fill);
            }
            if !stroke.is_empty() {
                painter.rect_stroke(box_rect, 0.0, stroke, StrokeKind::Middle);
            }
            let tcol = text_color.unwrap_or(style.text_color);
            if !text.is_empty() && tcol != Color32::TRANSPARENT {
                painter.text(
                    box_rect.left_top() + egui::vec2(5.0, 4.0),
                    Align2::LEFT_TOP,
                    text.as_ref(),
                    font.clone(),
                    tcol,
                );
            }
            Ok(true)
        }
        Annotation::Rectangle {
            coord,
            bounds,
            fill,
            stroke,
        } => {
            let stroke = valid_stroke(stroke.unwrap_or(style.border_stroke))?;
            let box_rect = resolve_box(*coord, *bounds, rect, transform)?;
            if *fill == Color32::TRANSPARENT && stroke.is_empty() {
                return Ok(false);
            }
            if *fill != Color32::TRANSPARENT {
                painter.rect_filled(box_rect, 0.0, *fill);
            }
            if !stroke.is_empty() {
                painter.rect_stroke(box_rect, 0.0, stroke, StrokeKind::Middle);
            }
            Ok(true)
        }
        Annotation::Ellipse {
            coord,
            bounds,
            fill,
            stroke,
        } => {
            let stroke = valid_stroke(stroke.unwrap_or(style.border_stroke))?;
            let box_rect = resolve_box(*coord, *bounds, rect, transform)?;
            if *fill == Color32::TRANSPARENT && stroke.is_empty() {
                return Ok(false);
            }
            let c = box_rect.center();
            let rx = box_rect.width() * 0.5;
            let ry = box_rect.height() * 0.5;
            let points: Vec<Pos2> = (0..48)
                .map(|i| {
                    let a = TAU * (i as f32 / 48.0);
                    egui::pos2(c.x + rx * a.cos(), c.y + ry * a.sin())
                })
                .collect();
            painter.add(Shape::Path(epaint::PathShape {
                points,
                closed: true,
                fill: *fill,
                stroke: epaint::PathStroke::new(stroke.width, stroke.color),
            }));
            Ok(true)
        }
        _ => Ok(false),
    }
}

fn valid_stroke(s: Stroke) -> Result<Stroke, ()> {
    (s.width.is_finite() && s.width >= 0.0)
        .then_some(s)
        .ok_or(())
}

fn valid_head(h: [f32; 2]) -> bool {
    h[0].is_finite() && h[1].is_finite() && h[0] >= 0.0 && h[1] >= 0.0
}

fn resolve_pt(
    coord: AnnotationCoord,
    p: PlotPoint,
    rect: Rect,
    transform: Option<PlotTransform>,
) -> Result<Pos2, ()> {
    if !p[0].is_finite() || !p[1].is_finite() {
        return Err(());
    }
    let pos = match coord {
        AnnotationCoord::Normalized => egui::pos2(
            rect.left() + p[0] as f32 * rect.width(),
            rect.bottom() - p[1] as f32 * rect.height(),
        ),
        AnnotationCoord::Data => {
            let t = transform.ok_or(())?;
            if !t.is_valid_point(p) {
                return Err(());
            }
            t.position_from_point(p)
        }
    };
    pos.is_finite().then_some(pos).ok_or(())
}

fn resolve_pair(
    coord: AnnotationCoord,
    a: PlotPoint,
    b: PlotPoint,
    rect: Rect,
    transform: Option<PlotTransform>,
) -> Result<(Pos2, Pos2), ()> {
    Ok((
        resolve_pt(coord, a, rect, transform)?,
        resolve_pt(coord, b, rect, transform)?,
    ))
}

fn resolve_box(
    coord: AnnotationCoord,
    bounds: [PlotPoint; 2],
    rect: Rect,
    transform: Option<PlotTransform>,
) -> Result<Rect, ()> {
    if !bounds[0][0].is_finite()
        || !bounds[0][1].is_finite()
        || !bounds[1][0].is_finite()
        || !bounds[1][1].is_finite()
        || bounds[1][0] <= bounds[0][0]
        || bounds[1][1] <= bounds[0][1]
    {
        return Err(());
    }
    let a = resolve_pt(coord, bounds[0], rect, transform)?;
    let b = resolve_pt(coord, bounds[1], rect, transform)?;
    let r = Rect::from_two_pos(a, b);
    (r.width() > 0.0 && r.height() > 0.0).then_some(r).ok_or(())
}

fn paint_arrowhead(painter: &Painter, from: Pos2, tip: Pos2, size: [f32; 2], stroke: Stroke) {
    if size[0] <= 0.0 || size[1] <= 0.0 {
        return;
    }
    let dx = tip.x - from.x;
    let dy = tip.y - from.y;
    let seg_len = dx.hypot(dy);
    if !seg_len.is_finite() || seg_len <= 1e-3 {
        return;
    }
    let scale = (0.4 * seg_len / size[0]).min(1.0);
    let len = size[0] * scale;
    let hw = size[1] * 0.5 * scale;
    let ux = dx / seg_len;
    let uy = dy / seg_len;
    let w1 = egui::pos2(tip.x - ux * len - uy * hw, tip.y - uy * len + ux * hw);
    let w2 = egui::pos2(tip.x - ux * len + uy * hw, tip.y - uy * len - ux * hw);
    if w1.is_finite() && w2.is_finite() {
        painter.add(Shape::line(vec![w1, tip, w2], stroke));
    }
}

fn tail_anchor(tail: Pos2, tip: Pos2) -> Align2 {
    let dx = tip.x - tail.x;
    let dy = tip.y - tail.y;
    if dx.abs() >= dy.abs() {
        if dx >= 0.0 {
            Align2::RIGHT_CENTER
        } else {
            Align2::LEFT_CENTER
        }
    } else if dy >= 0.0 {
        Align2::CENTER_BOTTOM
    } else {
        Align2::CENTER_TOP
    }
}
