#![allow(dead_code)] // Shared test drivers; each integration binary uses a subset.

use egui::{Context, Rect, Ui};
use egui_customplot::{CaptureLimits, ExportOptions, PlotSnapshot};

#[path = "input.rs"]
mod input;

pub fn snapshot(draw: impl FnOnce(&mut Ui) -> Rect) -> PlotSnapshot {
    let ctx = Context::default();
    let mut draw = Some(draw);
    let mut snapshot = None;
    ctx.run_ui(input::input(vec![]), |ui| {
        snapshot = Some(
            PlotSnapshot::capture(ui, CaptureLimits::default(), draw.take().unwrap()).unwrap(),
        );
    })
    .drop_without_applying_deltas();
    snapshot.unwrap()
}

pub fn decode_png(bytes: &[u8]) -> (png::OutputInfo, Vec<u8>) {
    let mut reader = png::Decoder::new(std::io::Cursor::new(bytes))
        .read_info()
        .unwrap();
    let mut rgba = vec![0; reader.output_buffer_size().unwrap()];
    let info = reader.next_frame(&mut rgba).unwrap();
    rgba.truncate(info.buffer_size());
    (info, rgba)
}

pub fn pixel(rgba: &[u8], width: usize, x: usize, y: usize) -> [u8; 4] {
    rgba[(y * width + x) * 4..][..4].try_into().unwrap()
}

pub fn render_svg(svg: &[u8], dimensions: [u32; 2]) -> Vec<u8> {
    let tree = resvg::usvg::Tree::from_data(svg, &resvg::usvg::Options::default()).unwrap();
    let mut image = resvg::tiny_skia::Pixmap::new(dimensions[0], dimensions[1]).unwrap();
    resvg::render(
        &tree,
        resvg::tiny_skia::Transform::identity(),
        &mut image.as_mut(),
    );
    image.data().to_vec() // premultiplied RGBA
}

pub fn png(snapshot: &PlotSnapshot) -> (png::OutputInfo, Vec<u8>) {
    decode_png(&snapshot.to_png(ExportOptions::default()).unwrap())
}
