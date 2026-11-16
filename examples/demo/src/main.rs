#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod appearance;
mod charts;
mod contours;
mod controls;
mod data;
mod diagnostics;
mod distributions;
mod export;
mod functions;
mod geometry;
mod multivariate;
mod networks;
mod performance;
mod plot3d;
mod polar;
mod polygons;
mod quiver;
mod scalar;
mod statistical;
mod stems;
mod stream;
mod surfaces3d;

fn main() -> eframe::Result {
    let gallery = std::env::args().nth(1);
    eframe::run_native(
        "egui-customplot | Stage 9B",
        eframe::NativeOptions {
            renderer: eframe::Renderer::Glow,
            viewport: eframe::egui::ViewportBuilder::default().with_inner_size([1200.0, 820.0]),
            ..Default::default()
        },
        Box::new(move |_| Ok(Box::new(app::Demo::with_gallery(gallery.as_deref())))),
    )
}
