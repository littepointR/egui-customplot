use std::fmt::{self, Write};

use egui::{Color32, Shape, Stroke, StrokeKind, epaint::ColorMode};

use super::writer::LimitedWriter;

/// Returns false without writing anything when raster fallback is required.
pub(super) fn write_shape(shape: &Shape, out: &mut LimitedWriter) -> Result<bool, fmt::Error> {
    match shape {
        Shape::LineSegment {
            points: [a, b],
            stroke,
        } if a.is_finite() && b.is_finite() => {
            write!(
                out,
                "<path d=\"M{} {}L{} {}\" fill=\"none\"",
                a.x, a.y, b.x, b.y
            )?;
            write_stroke(out, *stroke)?;
            out.write_str("/>")?;
        }
        Shape::Path(path)
            if path.stroke.kind == StrokeKind::Middle
                && matches!(path.stroke.color, ColorMode::Solid(_))
                && path.points.iter().all(|p| p.is_finite()) =>
        {
            out.write_str("<path d=\"")?;
            for (i, p) in path.points.iter().enumerate() {
                write!(out, "{}{} {}", if i == 0 { 'M' } else { 'L' }, p.x, p.y)?;
            }
            if path.closed {
                out.write_char('Z')?;
            }
            out.write_str("\"")?;
            write_color(out, "fill", path.fill)?;
            if let ColorMode::Solid(color) = path.stroke.color {
                write_stroke(out, Stroke::new(path.stroke.width, color))?;
            }
            out.write_str("/>")?;
        }
        Shape::Rect(r)
            if r.corner_radius == egui::CornerRadius::ZERO
                && r.angle == 0.0
                && r.blur_width == 0.0
                && r.brush.is_none()
                && r.rect.is_finite()
                && (r.stroke.is_empty() || r.rect.shrink(r.stroke.width).is_positive()) =>
        {
            write!(
                out,
                "<rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\"",
                r.rect.min.x,
                r.rect.min.y,
                r.rect.width(),
                r.rect.height()
            )?;
            write_color(out, "fill", r.fill)?;
            out.write_str("/>")?;
            if !r.stroke.is_empty() {
                let shift = match r.stroke_kind {
                    StrokeKind::Inside => -0.5,
                    StrokeKind::Middle => 0.0,
                    StrokeKind::Outside => 0.5,
                } * r.stroke.width;
                let rect = r.rect.expand(shift);
                write!(
                    out,
                    "<rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" fill=\"none\"",
                    rect.min.x,
                    rect.min.y,
                    rect.width(),
                    rect.height()
                )?;
                write_stroke(out, r.stroke)?;
                out.write_str("/>")?;
            }
        }
        Shape::Circle(circle) if circle.center.is_finite() && circle.radius.is_finite() => {
            write!(
                out,
                "<circle cx=\"{}\" cy=\"{}\" r=\"{}\"",
                circle.center.x, circle.center.y, circle.radius
            )?;
            write_color(out, "fill", circle.fill)?;
            write_stroke(out, circle.stroke)?;
            out.write_str("/>")?;
        }
        Shape::Mesh(mesh)
            if mesh.vertices.iter().all(|v| v.uv == egui::epaint::WHITE_UV)
                && mesh.triangles().all(|t| {
                    t.iter().all(|i| {
                        mesh.vertices[*i as usize].color == mesh.vertices[t[0] as usize].color
                    })
                }) =>
        {
            // No gradients are silently flattened. Solid numerical cells remain
            // vector triangles; varying-color marker AA takes the PNG fallback.
            out.write_str("<g shape-rendering=\"crispEdges\">")?;
            for triangle in mesh.triangles() {
                let [a, b, c] = triangle.map(|i| mesh.vertices[i as usize]);
                write!(
                    out,
                    "<path d=\"M{} {}L{} {}L{} {}Z\"",
                    a.pos.x, a.pos.y, b.pos.x, b.pos.y, c.pos.x, c.pos.y
                )?;
                write_color(out, "fill", a.color)?;
                out.write_str("/>")?;
            }
            out.write_str("</g>")?;
        }
        _ => return Ok(false),
    }
    Ok(true)
}

pub(super) fn write_color(out: &mut LimitedWriter, attribute: &str, color: Color32) -> fmt::Result {
    if color == Color32::TRANSPARENT {
        return write!(out, " {attribute}=\"none\"");
    }
    let [r, g, b, a] = color.to_srgba_unmultiplied();
    write!(
        out,
        " {attribute}=\"#{r:02x}{g:02x}{b:02x}\" {attribute}-opacity=\"{}\"",
        a as f32 / 255.0
    )
}

fn write_stroke(out: &mut LimitedWriter, stroke: Stroke) -> fmt::Result {
    if stroke.is_empty() {
        return out.write_str(" stroke=\"none\"");
    }
    write_color(out, "stroke", stroke.color)?;
    write!(
        out,
        " stroke-width=\"{}\" stroke-linejoin=\"round\" stroke-linecap=\"round\"",
        stroke.width
    )
}
