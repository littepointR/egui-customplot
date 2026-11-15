#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use eframe::egui;
use egui_customplot::{Line, Plot, PlotData, PlotState};

fn main() -> eframe::Result {
    eframe::run_native(
        "egui-customplot | Stage 1",
        eframe::NativeOptions {
            renderer: eframe::Renderer::Glow,
            viewport: egui::ViewportBuilder::default().with_inner_size([1100.0, 760.0]),
            ..Default::default()
        },
        Box::new(|_| Ok(Box::new(Demo::default()))),
    )
}

struct Demo {
    data: PlotData,
    state: PlotState,
    detail_state: PlotState,
    count: usize,
    gaps: bool,
    second_view: bool,
    grid: bool,
    width: f32,
    color: egui::Color32,
}

impl Default for Demo {
    fn default() -> Self {
        Self {
            data: make_data(10_000, true),
            state: PlotState::default(),
            detail_state: PlotState::default(),
            count: 10_000,
            gaps: true,
            second_view: false,
            grid: true,
            width: 1.5,
            color: egui::Color32::from_rgb(70, 160, 235),
        }
    }
}

fn make_data(count: usize, gaps: bool) -> PlotData {
    PlotData::new(
        (0..count)
            .map(|i| {
                let x = i as f64 / (count - 1) as f64 * 20.0;
                let y = if gaps && (8.0..8.7).contains(&x) {
                    f64::NAN
                } else {
                    x.sin() + 0.12 * (13.0 * x).sin()
                };
                [x, y]
            })
            .collect::<Vec<_>>(),
    )
}

impl eframe::App for Demo {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ui, |ui| {
            ui.heading("egui-customplot / Stage 1 prototype");
            ui.label("Drag: pan   |   Hover + wheel: zoom   |   Double-click / focused Home: auto-fit");
            ui.horizontal(|ui| {
                let old = (self.count, self.gaps);
                egui::ComboBox::from_id_salt("samples").selected_text(format!("{} points", self.count)).show_ui(ui, |ui| {
                    for n in [1_000, 10_000, 100_000, 1_000_000] { ui.selectable_value(&mut self.count, n, format!("{n} points")); }
                });
                ui.checkbox(&mut self.gaps, "NaN gap");
                if old != (self.count, self.gaps) { self.data = make_data(self.count, self.gaps); }
                if ui.button("Auto-fit both").clicked() { self.state.reset(); self.detail_state.reset(); }
                ui.checkbox(&mut self.grid, "Grid");
                ui.checkbox(&mut self.second_view, "Independent second view");
                egui::widgets::global_theme_preference_buttons(ui);
            });
            ui.horizontal(|ui| {
                ui.color_edit_button_srgba(&mut self.color);
                ui.add(egui::Slider::new(&mut self.width, 0.5..=5.0).text("Line width"));
                ui.label("Shared immutable data; no LOD or geometry cache yet.");
            });
            let height = if self.second_view { (ui.available_height() - 110.0) / 2.0 }
                else { ui.available_height() - 85.0 };
            let lines = [Line::new(&self.data).stroke((self.width, self.color))];
            let out = Plot::new("main-plot").height(height.max(80.0)).show_grid(self.grid)
                .allow_scroll_zoom(true).show(ui, &mut self.state, &lines);
            let d = out.diagnostics;
            ui.monospace(format!("samples: {}  |  scanned segments: {}  |  output: {} points / {} paths",
                d.input_points, d.visited_segments, d.output_points, d.output_paths));
            ui.monospace(format!("sample payload: {:.2} MiB  |  screen Vec capacity: {:.2} MiB  |  prepare+submit: {:.3} ms",
                d.unique_sample_bytes as f64 / 1_048_576.0, d.geometry_capacity_bytes as f64 / 1_048_576.0,
                d.prepare_duration.as_secs_f64() * 1000.0));
            ui.small("Per-pass CPU counters only; excludes egui tessellation, driver/GPU memory and presentation. Static plots do not request continuous repaint.");
            if self.second_view {
                Plot::new("detail-plot").height(height.max(80.0)).show_grid(self.grid)
                    .allow_scroll_zoom(true).show(ui, &mut self.detail_state, &lines);
            }
        });
    }
}
