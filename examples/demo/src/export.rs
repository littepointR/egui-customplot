//! Deferred disk effects: discarded passes only replace a snapshot candidate.
use std::{io::Write, path::PathBuf};

use eframe::egui;
use egui_customplot::{ExportOptions, PlotSnapshot};

use crate::i18n::{Language, lang, tr};

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
        let l = lang(ui.ctx());
        ui.separator();
        ui.label(tr(
            l,
            "Export displayed gallery",
            "导出当前展示图表",
            "表示中ギャラリーを書出",
        ));
        ui.text_edit_singleline(&mut self.path);
        ui.add(
            egui::Slider::new(&mut self.pixels_per_point, 0.5..=3.0).text(tr(
                l,
                "Pixels / point",
                "像素 / 逻辑点",
                "px / pt",
            )),
        );
        ui.checkbox(
            &mut self.transparent,
            tr(
                l,
                "Transparent outer background",
                "外层透明背景",
                "外側背景を透過",
            ),
        );
        let frame = ui.ctx().cumulative_frame_nr();
        ui.horizontal(|ui| {
            for (label, format) in [
                (tr(l, "Save PNG", "保存 PNG", "PNG 保存"), Format::Png),
                (
                    tr(
                        l,
                        "Save hybrid SVG",
                        "保存混合 SVG",
                        "ハイブリッド SVG 保存",
                    ),
                    Format::Svg,
                ),
            ] {
                if ui.button(label).clicked() && self.last_click != Some(frame) {
                    self.request = Some(format);
                    self.candidate = None;
                    self.last_click = Some(frame);
                }
            }
        });
        ui.small(tr(
            l,
            "Captures displayed sampling (MinMax stays approximate). Fonts and AA mesh are embedded pixels in SVG. Scale does not relayout text. No file overwrite.",
            "截取当前已渲染采样（含 MinMax）。字体与抗锯齿网格在 SVG 中内嵌位图，拒绝覆盖已有文件。",
            "現在表示中のサンプリングを書出（MinMax は近似）。SVG 内の文字・AA は埋め込み画像で上書き禁止です。",
        ));
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
            Ok(path) => match lang(ctx) {
                Language::English => format!("Saved {}", path.display()),
                Language::ZhCn => format!("已保存 {}", path.display()),
                Language::Japanese => format!("保存完了: {}", path.display()),
            },
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
