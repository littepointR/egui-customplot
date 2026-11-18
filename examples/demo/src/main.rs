#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod appearance;
mod charts;
mod contours;
mod controls;
mod custom_animation;
mod custom_annotations;
mod custom_axes;
mod custom_styling;
mod data;
mod diagnostics;
mod distributions;
mod dynamic10m;
mod export;
mod fonts;
mod functions;
mod gallery;
mod geo;
mod geometry;
mod global_controls;
mod i18n;
mod multivariate;
mod networks;
mod os_theme;
mod performance;
mod plot3d;
mod polar;
mod polygons;
mod quiver;
mod scalar;
mod stage13;
mod stage14;
mod stage15;
mod stage16;
mod statistical;
mod stems;
mod stream;
mod surfaces3d;
mod time_measurements;
mod volumes3d;

fn main() -> eframe::Result {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let initial_lang = i18n::Language::detect_default(&args);
    let use_glow = args.iter().any(|a| a == "--glow");
    let bench_csv = args
        .iter()
        .find_map(|a| a.strip_prefix("--bench-window-10m="))
        .map(std::path::PathBuf::from);
    let uncapped = args
        .iter()
        .any(|a| a == "--uncapped" || a == "--dynamic-10m")
        || bench_csv.is_some();
    let gallery = args
        .iter()
        .find(|a| {
            *a != "--glow"
                && *a != "--uncapped"
                && !a.starts_with("--bench-window-10m=")
                && !i18n::is_lang_arg(a)
        })
        .cloned();

    let mut wgpu_options = eframe::egui_wgpu::WgpuConfiguration::default();
    if let eframe::egui_wgpu::WgpuSetup::CreateNew(setup) = &mut wgpu_options.wgpu_setup {
        setup.power_preference = eframe::wgpu::PowerPreference::HighPerformance;
    }
    if uncapped {
        wgpu_options.surface = eframe::SurfaceConfig {
            present_mode: eframe::wgpu::PresentMode::AutoNoVsync,
            desired_maximum_frame_latency: Some(1),
        };
    }

    eframe::run_native(
        initial_lang.window_title(),
        eframe::NativeOptions {
            renderer: if use_glow {
                eframe::Renderer::Glow
            } else {
                eframe::Renderer::Wgpu
            },
            wgpu_options,
            viewport: eframe::egui::ViewportBuilder::default().with_inner_size([1200.0, 820.0]),
            ..Default::default()
        },
        Box::new(move |cc| {
            let mut demo = app::Demo::with_gallery(gallery.as_deref());
            demo.set_language(initial_lang);
            demo.install_os_theme_sync(&cc.egui_ctx);
            fonts::ensure_cjk_fonts(&cc.egui_ctx);
            i18n::sync_context(&cc.egui_ctx, initial_lang);
            if let Some(rs) = cc.wgpu_render_state.as_ref() {
                let registered = egui_customplot::register_wgpu(&cc.egui_ctx, rs);
                let info = rs.adapter.get_info();
                demo.set_wgpu_info(registered, format!("{} ({:?})", info.name, info.backend));
            }
            if let Some(csv) = bench_csv.clone() {
                demo.configure_window_bench(csv);
            }
            Ok(Box::new(demo))
        }),
    )
}
