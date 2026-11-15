use std::fmt::Write;

use egui::{Color32, Rect};

use super::{
    ExportError, ExportOptions, PlotSnapshot, raster, vector,
    writer::{LimitedWriter, base64},
};

pub(super) fn encode(scene: &PlotSnapshot, options: ExportOptions) -> Result<Vec<u8>, ExportError> {
    let dimensions = options.dimensions(scene.rect)?;
    let mut output = LimitedWriter::new(options.max_output_bytes);
    let rect = scene.rect;
    let logical_size =
        egui::vec2(dimensions[0] as f32, dimensions[1] as f32) / options.pixels_per_point;
    write!(output, "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{}\" height=\"{}\" viewBox=\"{} {} {} {}\">",
        dimensions[0], dimensions[1], rect.min.x, rect.min.y, logical_size.x, logical_size.y).map_err(limit)?;
    output.append(b"<desc>Displayed sampling. Hybrid vector geometry with embedded host-font and varying-color raster layers. No external resources.</desc>")?;
    write!(
        output,
        "<rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\"",
        rect.min.x, rect.min.y, logical_size.x, logical_size.y
    )
    .map_err(limit)?;
    vector::write_color(&mut output, "fill", options.background).map_err(limit)?;
    output.append(b"/>")?;
    let mut steps = 0;
    for (index, item) in scene.items.iter().enumerate() {
        let clip = item.clip.intersect(rect);
        if !clip.is_positive() {
            continue;
        }
        write!(output, "<defs><clipPath id=\"clip{index}\" clipPathUnits=\"userSpaceOnUse\"><rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\"/></clipPath></defs><g clip-path=\"url(#clip{index})\">",
            clip.min.x, clip.min.y, clip.width(), clip.height()).map_err(limit)?;
        if !vector::write_shape(&item.shape, &mut output).map_err(limit)? {
            let bounds = item
                .meshes
                .iter()
                .fold(Rect::NOTHING, |r, m| r.union(m.calc_bounds()))
                .intersect(clip);
            if bounds.is_positive() {
                // Align PNG patches to a single output pixel grid. Text rotation,
                // color overrides and underline were already applied by egui.
                let ppp = options.pixels_per_point;
                let min = rect.min + ((bounds.min - rect.min) * ppp).floor() / ppp;
                let max = rect.min + ((bounds.max - rect.min) * ppp).ceil() / ppp;
                let region = Rect::from_min_max(min, max);
                let size = options.dimensions(region)?;
                let transparent = ExportOptions {
                    background: Color32::TRANSPARENT,
                    ..options
                };
                let rgba = raster::render_items(
                    scene,
                    std::slice::from_ref(item),
                    region,
                    size,
                    transparent,
                    &mut steps,
                )?;
                let png = super::png::encode(&rgba, size, transparent)?;
                write!(output, "<image x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" preserveAspectRatio=\"none\" href=\"data:image/png;base64,",
                    region.min.x, region.min.y, region.width(), region.height()).map_err(limit)?;
                base64(&png, &mut output)?;
                output.append(b"\"/>")?;
            }
        }
        output.append(b"</g>")?;
    }
    output.append(b"</svg>\n")?;
    Ok(output.bytes)
}

fn limit(_: std::fmt::Error) -> ExportError {
    ExportError::LimitExceeded("encoded output bytes")
}
