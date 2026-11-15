use super::{ExportError, ExportOptions, writer::LimitedWriter};

pub(super) fn encode(
    pixels: &[u8],
    size: [usize; 2],
    options: ExportOptions,
) -> Result<Vec<u8>, ExportError> {
    let mut output = LimitedWriter::new(options.max_output_bytes);
    {
        let mut encoder = png::Encoder::new(&mut output, size[0] as u32, size[1] as u32);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        encoder.set_source_srgb(png::SrgbRenderingIntent::Perceptual);
        let pixels_per_meter = (options.dpi as f64 / 0.0254).round() as u32;
        encoder.set_pixel_dims(Some(png::PixelDimensions {
            xppu: pixels_per_meter,
            yppu: pixels_per_meter,
            unit: png::Unit::Meter,
        }));
        encoder.add_text_chunk("Description".into(), "Displayed egui geometry; sampling and host font atlas captured, not a raw-data rerender.".into())
            .map_err(encoding)?;
        let mut writer = encoder.write_header().map_err(encoding)?;
        writer.write_image_data(pixels).map_err(encoding)?;
        writer.finish().map_err(encoding)?;
    }
    Ok(output.bytes)
}

fn encoding(error: png::EncodingError) -> ExportError {
    ExportError::Encoding(error.to_string())
}
