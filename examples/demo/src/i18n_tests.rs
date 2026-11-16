use super::*;
use crate::i18n::{Language, is_lang_arg, tr};

const GALLERIES: [Option<&str>; 19] = [
    None,
    Some("--scalar"),
    Some("--statistical"),
    Some("--stairs"),
    Some("--stems"),
    Some("--quiver"),
    Some("--polygons"),
    Some("--contours"),
    Some("--functions"),
    Some("--geometry"),
    Some("--distributions"),
    Some("--multivariate"),
    Some("--polar"),
    Some("--networks"),
    Some("--plot3d"),
    Some("--surfaces3d"),
    Some("--volumes3d"),
    Some("--geo"),
    Some("--dynamic-10m"),
];

#[test]
fn cli_and_env_language_parsing_works() {
    assert_eq!(Language::default(), Language::English);
    assert_eq!(Language::from_token("zh-CN"), Some(Language::ZhCn));
    assert_eq!(Language::from_token("zh_Hans"), Some(Language::ZhCn));
    assert_eq!(
        Language::from_token("ja_JP.UTF-8"),
        Some(Language::Japanese)
    );
    assert_eq!(Language::from_token("en-US"), Some(Language::English));

    let zh_args = vec!["customplot-demo".into(), "--zh".into(), "--geo".into()];
    assert_eq!(Language::from_args(&zh_args), Some(Language::ZhCn));

    let ja_args = vec!["customplot-demo".into(), "--lang=ja".into()];
    assert_eq!(Language::from_args(&ja_args), Some(Language::Japanese));

    assert!(is_lang_arg("--lang=zh"));
    assert!(is_lang_arg("--en"));
    assert!(is_lang_arg("--zh"));
    assert!(is_lang_arg("--ja"));
    assert!(!is_lang_arg("--geo"));

    for lang in Language::ALL {
        assert!(
            lang.window_title().starts_with("egui-customplot"),
            "{:?} window title must preserve smoke-test prefix",
            lang
        );
    }
    assert_eq!(tr(Language::English, "a", "b", "c"), "a");
    assert_eq!(tr(Language::ZhCn, "a", "b", "c"), "b");
    assert_eq!(tr(Language::Japanese, "a", "b", "c"), "c");
}

#[test]
fn system_cjk_font_loader_succeeds_idempotently() {
    let ctx = egui::Context::default();
    let first = crate::fonts::ensure_cjk_fonts(&ctx);
    let second = crate::fonts::ensure_cjk_fonts(&ctx);
    if cfg!(target_os = "windows") {
        assert!(first, "expected a system CJK font to load on Windows");
    }
    assert_eq!(first, second);

    let out = ctx.run_ui(input(0.0), |ui| {
        ui.label("简体中文 日本語 English Title 123");
    });
    assert!(!out.shapes.is_empty());
    out.drop_without_applying_deltas();
}

#[test]
fn all_galleries_render_across_all_languages_and_multipass() {
    let mut time = 0.0;
    for lang in Language::ALL {
        let ctx = egui::Context::default();
        ctx.options_mut(|o| o.max_passes = std::num::NonZeroUsize::new(4).unwrap());
        crate::i18n::sync_context(&ctx, lang);
        for gallery in GALLERIES {
            let mut demo = Demo::with_gallery(gallery);
            demo.set_language(lang);
            demo.dynamic10m.set_capacity_for_test(4_096);
            if gallery != Some("--stairs") {
                demo.options.count = 1000;
                demo.data = DemoData::new(demo.options.scene, 1000, false);
            }
            let mut passes = 0;
            let out = ctx.run_ui(input(time), |ui| {
                demo.show(ui);
                if passes == 0 {
                    ui.ctx().request_discard("i18n multipass");
                }
                passes += 1;
            });
            assert_eq!(passes, 2, "{lang:?} {gallery:?}");
            assert!(!out.shapes.is_empty(), "{lang:?} {gallery:?}");
            out.drop_without_applying_deltas();
            time += 0.1;
        }
    }
}

#[test]
fn live_language_switching_preserves_view_bounds_and_data_versions() {
    let ctx = egui::Context::default();
    let mut demo = Demo::default();
    let manual = egui_customplot::PlotBounds::new([1.5, -0.8], [8.5, 0.9]).unwrap();
    demo.views.main.set_bounds(manual);
    let signal_version = demo.data.signal.version();

    let mut time = 0.0;
    for lang in [
        Language::English,
        Language::ZhCn,
        Language::Japanese,
        Language::English,
    ] {
        demo.set_language(lang);
        let out = ctx.run_ui(input(time), |ui| demo.show(ui));
        assert!(!out.shapes.is_empty());
        out.drop_without_applying_deltas();
        assert_eq!(demo.views.main.bounds(), Some(manual));
        assert_eq!(demo.data.signal.version(), signal_version);
        time += 0.1;
    }

    for (idx, (lang, prefix)) in [(Language::ZhCn, "已保存"), (Language::Japanese, "保存完了")]
        .into_iter()
        .enumerate()
    {
        let base = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(format!(
            "../../target/export-test-output/demo-i18n-{}-{idx}",
            std::process::id()
        ));
        let path = base.with_extension("png");
        let _ = std::fs::remove_file(&path);
        demo.set_language(lang);
        demo.export.path = base.to_string_lossy().into_owned();
        demo.export.pixels_per_point = 0.5;
        demo.export.request = Some(crate::export::Format::Png);
        ctx.run_ui(input(time), |ui| demo.show(ui))
            .drop_without_applying_deltas();
        time += 0.1;
        ctx.run_ui(input(time), |ui| demo.show(ui))
            .drop_without_applying_deltas();
        time += 0.1;
        assert!(
            demo.export.status.starts_with(prefix),
            "{lang:?} status={}",
            demo.export.status
        );
        assert!(path.exists());
        std::fs::remove_file(path).unwrap();
    }
}

#[test]
fn cjk_localized_galleries_export_self_contained_svg_and_png() {
    for lang in [Language::ZhCn, Language::Japanese] {
        let ctx = egui::Context::default();
        crate::i18n::sync_context(&ctx, lang);
        let mut demo = Demo::with_gallery(None);
        demo.set_language(lang);
        demo.options.count = 1000;
        demo.data = DemoData::new(demo.options.scene, 1000, false);

        let mut snapshot = None;
        ctx.run_ui(input(0.0), |ui| {
            snapshot = Some(
                egui_customplot::PlotSnapshot::capture(ui, Default::default(), |ui| {
                    demo.show_charts(ui)
                        .iter()
                        .fold(egui::Rect::NOTHING, |rect, r| rect.union(r.outer_rect))
                })
                .unwrap(),
            );
        })
        .drop_without_applying_deltas();
        drop((demo, ctx));

        let options = egui_customplot::ExportOptions {
            pixels_per_point: 0.5,
            ..Default::default()
        };
        let snapshot = snapshot.unwrap();
        let svg = String::from_utf8(snapshot.to_svg(options).unwrap()).unwrap();
        assert!(
            svg.contains("<svg")
                && svg.contains("clipPath")
                && svg.contains("data:image/png;base64,")
        );
        assert!(!svg.contains("<text"));
        assert!(snapshot.to_png(options).unwrap().starts_with(b"\x89PNG"));
    }
}
