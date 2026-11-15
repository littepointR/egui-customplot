//! A standalone headless HOST using the same public API as a native egui app.
use std::{io::Write, path::PathBuf};

use egui::{Color32, Context, RawInput, Rect, pos2, vec2};
use egui_customplot::{
    Axis, CaptureLimits, ColorScale, ErrorBar, ErrorBarData, ErrorBars, ExportOptions, GridData,
    Heatmap, Line, LineSampling, Plot, PlotData, PlotSnapshot, PlotState,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let base = PathBuf::from(
        std::env::args()
            .nth(1)
            .unwrap_or_else(|| "target/exports/example".into()),
    );
    let ctx = Context::default();
    let data = PlotData::new(
        (0..=256)
            .map(|i| {
                let x = i as f64 / 32.0;
                [x, x.sin()]
            })
            .collect::<Vec<_>>(),
    );
    let grid = GridData::new(
        vec![0.0, 2.0, 4.0, 6.0, 8.0],
        vec![-1.5, 0.0, 1.5],
        vec![0.1, 0.3, f64::NAN, 0.6, 0.3, 0.5, 0.7, 1.0],
    )?;
    let errors = ErrorBarData::new(vec![
        ErrorBar::new([2.0, 2.0f64.sin()]).y_error([0.25, 0.4]),
    ]);
    let mut state = PlotState::default();
    let mut snapshot = None;
    ctx.run_ui(
        RawInput {
            screen_rect: Some(Rect::from_min_size(pos2(0.0, 0.0), vec2(800.0, 500.0))),
            ..Default::default()
        },
        |ui| {
            snapshot = Some(PlotSnapshot::capture(ui, CaptureLimits::default(), |ui| {
                Plot::new("export-example")
                    .height(480.0)
                    .title("Displayed exact geometry / embedded host fonts")
                    .x_axis(Axis::new().label("Time (s)"))
                    .y_axis(Axis::new().label("Amplitude"))
                    .show_layers(
                        ui,
                        &mut state,
                        &[
                            Heatmap::new(&grid, ColorScale::new([0.0, 1.0]).unwrap()).into(),
                            Line::new(&data)
                                .sampling(LineSampling::Exact)
                                .color(Color32::WHITE)
                                .name("Sine")
                                .into(),
                            ErrorBars::new(&errors)
                                .color(Color32::RED)
                                .name("Offsets")
                                .into(),
                        ],
                    )
                    .outer_rect
            }));
        },
    )
    .drop_without_applying_deltas();
    let snapshot = snapshot.unwrap()?;
    drop((ctx, state, data, grid, errors)); // encoding does not retain the host
    for (extension, bytes) in [
        ("png", snapshot.to_png(ExportOptions::default())?),
        ("svg", snapshot.to_svg(ExportOptions::default())?),
    ] {
        let file = base.with_extension(extension);
        if let Some(parent) = file.parent().filter(|p| !p.as_os_str().is_empty()) {
            std::fs::create_dir_all(parent)?;
        }
        let mut writer = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&file)?;
        writer.write_all(&bytes)?;
        println!(
            "saved={}; bytes={}; source_ppp={}; sampling=displayed Exact",
            file.display(),
            bytes.len(),
            snapshot.source_pixels_per_point()
        );
    }
    Ok(())
}
