//! Deferred disk effects: discarded passes only replace a snapshot candidate.
use std::{io::Write, path::PathBuf};

use eframe::egui;
use egui_customplot::{ExportOptions, PlotSnapshot};

#[derive(Clone, Copy)]
pub(super) enum Format {
    Png,
    Svg,
}

impl Format {
    fn extension(self) -> &'static str {
        match self {
            Self::Png => "png",
            Self::Svg => "svg",
        }
    }
}

pub(super) struct ExportPanel {
    pub path: String,
    pub pixels_per_point: f32,
    pub transparent: bool,
    pub request: Option<Format>,
    candidate: Option<(u64, Result<PlotSnapshot, String>)>,
    last_click: Option<u64>,
    pub status: String,
}

impl Default for ExportPanel {
    fn default() -> Self {
        Self {
            path: "target/exports/plot".into(),
            pixels_per_point: 1.0,
            transparent: true,
            request: None,
            candidate: None,
            last_click: None,
            status: String::new(),
        }
    }
}

impl ExportPanel {
    pub fn controls(&mut self, ui: &mut egui::Ui) {
        ui.separator();
        ui.label("Export displayed gallery");
        ui.text_edit_singleline(&mut self.path);
        ui.add(egui::Slider::new(&mut self.pixels_per_point, 0.5..=3.0).text("Pixels / point"));
        ui.checkbox(&mut self.transparent, "Transparent outer background");
        let frame = ui.ctx().cumulative_frame_nr();
        ui.horizontal(|ui| {
            for (label, format) in [("Save PNG", Format::Png), ("Save hybrid SVG", Format::Svg)] {
                if ui.button(label).clicked() && self.last_click != Some(frame) {
                    self.request = Some(format);
                    self.candidate = None;
                    self.last_click = Some(frame);
                }
            }
        });
        ui.small("Captures displayed sampling (MinMax stays approximate). Fonts and AA mesh are embedded pixels in SVG. Scale does not relayout text. No file overwrite.");
        if !self.status.is_empty() {
            ui.small(&self.status);
        }
    }

    pub fn store(
        &mut self,
        ctx: &egui::Context,
        result: Result<PlotSnapshot, egui_customplot::ExportError>,
    ) {
        self.candidate = Some((ctx.cumulative_frame_nr(), result.map_err(|e| e.to_string())));
        ctx.request_repaint();
    }

    /// A new frame implies all previous layout passes have finished. No file is
    /// created by a discarded pass; the last candidate replaces the earlier one.
    pub fn flush_previous(&mut self, ctx: &egui::Context) {
        if self
            .candidate
            .as_ref()
            .is_none_or(|(frame, _)| *frame == ctx.cumulative_frame_nr())
        {
            return;
        }
        let (_, candidate) = self.candidate.take().unwrap();
        let Some(format) = self.request.take() else {
            return;
        };
        let options = ExportOptions {
            pixels_per_point: self.pixels_per_point,
            background: if self.transparent {
                egui::Color32::TRANSPARENT
            } else {
                egui::Color32::WHITE
            },
            ..Default::default()
        };
        let result = candidate.and_then(|snapshot| save(&snapshot, format, &self.path, options));
        self.status = match result {
            Ok(path) => format!("Saved {}", path.display()),
            Err(e) => e,
        };
    }
}

fn save(
    snapshot: &PlotSnapshot,
    format: Format,
    path: &str,
    options: ExportOptions,
) -> Result<PathBuf, String> {
    let bytes = match format {
        Format::Png => snapshot.to_png(options),
        Format::Svg => snapshot.to_svg(options),
    }
    .map_err(|e| e.to_string())?;
    let path = PathBuf::from(format!("{path}.{}", format.extension()));
    if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
        .map_err(|e| e.to_string())?;
    file.write_all(&bytes).map_err(|e| e.to_string())?;
    Ok(path)
}
