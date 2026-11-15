#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod controls;
mod data;
mod diagnostics;

fn main() -> eframe::Result {
    eframe::run_native(
        "egui-customplot | Stage 2",
        eframe::NativeOptions {
            renderer: eframe::Renderer::Glow,
            viewport: eframe::egui::ViewportBuilder::default().with_inner_size([1200.0, 820.0]),
            ..Default::default()
        },
        Box::new(|_| Ok(Box::new(app::Demo::default()))),
    )
}
