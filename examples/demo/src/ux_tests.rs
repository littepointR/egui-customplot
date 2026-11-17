use super::*;

fn collect_shapes<'a>(shape: &'a egui::epaint::Shape, out: &mut Vec<&'a egui::epaint::Shape>) {
    match shape {
        egui::epaint::Shape::Vec(children) => {
            for child in children {
                collect_shapes(child, out);
            }
        }
        other => out.push(other),
    }
}

#[test]
fn animations_advance_by_wall_time_not_pointer_event_frame_rate() {
    // Compare 5 idle frames (dt = 0.076s, including a multipass discard) vs
    // 190 pointer-spam frames (dt = 0.002s + PointerMoved events); both span 0.38s.
    let mut idle_p3d = crate::plot3d::Plot3DDemo::default();
    let mut spam_p3d = crate::plot3d::Plot3DDemo::default();
    idle_p3d.animate_comet = true;
    spam_p3d.animate_comet = true;

    let mut idle_vol = crate::volumes3d::Volumes3DDemo::default();
    let mut spam_vol = crate::volumes3d::Volumes3DDemo::default();
    idle_vol.animate_particles = true;
    spam_vol.animate_particles = true;

    let mut idle_fn = crate::functions::FunctionsDemo::default();
    let mut spam_fn = crate::functions::FunctionsDemo::default();
    idle_fn.animate_live = true;
    spam_fn.animate_live = true;

    let idle_ctx = egui::Context::default();
    let spam_ctx = egui::Context::default();

    // Warmup frame at t = 0.0 initializes clocks without advancing dt.
    idle_ctx
        .run_ui(input(0.0), |ui| {
            let _ = idle_p3d.show(ui);
            let _ = idle_vol.show(ui);
            let _ = idle_fn.show(ui);
        })
        .drop_without_applying_deltas();
    spam_ctx
        .run_ui(input(0.0), |ui| {
            let _ = spam_p3d.show(ui);
            let _ = spam_vol.show(ui);
            let _ = spam_fn.show(ui);
        })
        .drop_without_applying_deltas();

    let p3d_start = idle_p3d.comet_progress();
    let vol_start = idle_vol.particle_phase();
    let fn_start = idle_fn.comet_progress();

    for step in 1..=5 {
        let t = step as f64 * 0.076;
        let discarded_once = std::cell::Cell::new(false);
        idle_ctx
            .run_ui(input(t), |ui| {
                if step == 2 && !discarded_once.replace(true) {
                    ui.ctx().request_discard("multipass-frameclock-test");
                }
                let _ = idle_p3d.show(ui);
                let _ = idle_vol.show(ui);
                let _ = idle_fn.show(ui);
            })
            .drop_without_applying_deltas();
    }

    for step in 1..=190 {
        let mut raw = input(step as f64 * 0.002);
        raw.events.push(egui::Event::PointerMoved(egui::pos2(
            80.0 + (step % 40) as f32,
            90.0,
        )));
        spam_ctx
            .run_ui(raw, |ui| {
                let _ = spam_p3d.show(ui);
                let _ = spam_vol.show(ui);
                let _ = spam_fn.show(ui);
            })
            .drop_without_applying_deltas();
    }

    assert!(
        (idle_p3d.comet_progress() - p3d_start).abs() > 1e-4,
        "Plot3D comet did not advance"
    );
    assert!(
        (idle_vol.particle_phase() - vol_start).abs() > 1e-4,
        "Volumes3D particles did not advance"
    );
    assert!(
        (idle_fn.comet_progress() - fn_start).abs() > 1e-4,
        "Functions comet did not advance"
    );

    assert!(
        (idle_p3d.comet_progress() - spam_p3d.comet_progress()).abs() < 1e-6,
        "Plot3D animation speed depended on frame rate: idle={}, spam={}",
        idle_p3d.comet_progress(),
        spam_p3d.comet_progress()
    );
    assert!(
        (idle_vol.particle_phase() - spam_vol.particle_phase()).abs() < 1e-6,
        "Volumes3D animation speed depended on frame rate: idle={}, spam={}",
        idle_vol.particle_phase(),
        spam_vol.particle_phase()
    );
    assert!(
        (idle_fn.comet_progress() - spam_fn.comet_progress()).abs() < 1e-6,
        "Functions animation speed depended on frame rate: idle={}, spam={}",
        idle_fn.comet_progress(),
        spam_fn.comet_progress()
    );
}

#[test]
fn theme_preference_selector_switches_modes_using_vector_shapes() {
    let ctx = egui::Context::default();
    let mut time = 0.0;
    let render_and_locate = |time: &mut f64, target_label: &str| -> egui::Pos2 {
        let out = ctx.run_ui(input(*time), |ui| {
            crate::i18n::theme_preference_selector(ui, crate::i18n::Language::ZhCn);
        });
        *time += 0.05;

        let mut flat = Vec::new();
        for clipped in &out.shapes {
            collect_shapes(&clipped.shape, &mut flat);
        }

        let mut has_circle = false;
        let mut has_line = false;
        let mut has_mesh = false;
        let mut label_center = None;

        for shape in flat {
            match shape {
                egui::epaint::Shape::Circle(_) => has_circle = true,
                egui::epaint::Shape::LineSegment { .. } => has_line = true,
                egui::epaint::Shape::Mesh(_) => has_mesh = true,
                egui::epaint::Shape::Text(text) => {
                    let s = text.galley.text();
                    assert!(
                        !s.chars().any(|c| matches!(
                            c,
                            '\u{2600}' | '\u{1f319}' | '\u{1f4bb}' | '\u{1f317}'
                        )),
                        "unexpected emoji character in theme selector label: {s:?}"
                    );
                    if s == target_label {
                        label_center = Some(text.pos + text.galley.size() * 0.5);
                    }
                }
                _ => {}
            }
        }
        out.drop_without_applying_deltas();

        assert!(
            has_circle && has_line && has_mesh,
            "expected vector theme icons"
        );
        label_center.unwrap_or_else(|| panic!("missing button label {target_label:?}"))
    };

    for (label, expected) in [
        ("深色", egui::ThemePreference::Dark),
        ("浅色", egui::ThemePreference::Light),
        ("跟随", egui::ThemePreference::System),
    ] {
        let pos = render_and_locate(&mut time, label);
        let mut press = input(time);
        press.events.push(egui::Event::PointerMoved(pos));
        press.events.push(egui::Event::PointerButton {
            pos,
            button: egui::PointerButton::Primary,
            pressed: true,
            modifiers: egui::Modifiers::NONE,
        });
        ctx.run_ui(press, |ui| {
            crate::i18n::theme_preference_selector(ui, crate::i18n::Language::ZhCn);
        })
        .drop_without_applying_deltas();
        time += 0.05;

        let mut release = input(time);
        release.events.push(egui::Event::PointerMoved(pos));
        release.events.push(egui::Event::PointerButton {
            pos,
            button: egui::PointerButton::Primary,
            pressed: false,
            modifiers: egui::Modifiers::NONE,
        });
        ctx.run_ui(release, |ui| {
            crate::i18n::theme_preference_selector(ui, crate::i18n::Language::ZhCn);
        })
        .drop_without_applying_deltas();
        time += 0.05;

        assert_eq!(
            ctx.options(|o| o.theme_preference),
            expected,
            "clicking {label:?} did not update theme_preference"
        );
    }
}

#[test]
fn gallery_selector_switches_mutually_exclusively_across_all_27_views() {
    use crate::gallery::ActiveGallery;

    assert_eq!(ActiveGallery::ALL.len(), 27);
    assert_eq!(
        ActiveGallery::Default2D.prev().next(),
        ActiveGallery::Default2D
    );

    let mut demo = Demo::default();
    let orig_interp = demo.options.interpolation;
    let orig_count = demo.options.count;

    demo.select_gallery(ActiveGallery::Stairs);
    assert_eq!(demo.gallery, ActiveGallery::Stairs);
    assert_ne!(demo.options.interpolation, orig_interp);

    demo.select_gallery(ActiveGallery::Default2D);
    assert_eq!(demo.gallery, ActiveGallery::Default2D);
    assert_eq!(demo.options.interpolation, orig_interp);
    assert_eq!(demo.options.count, orig_count);

    let ctx = egui::Context::default();
    let mut time = 0.0;
    let mut current = ActiveGallery::Default2D;
    for expected in ActiveGallery::ALL {
        demo.select_gallery(expected);
        assert_eq!(demo.gallery, expected);
        let output = ctx.run_ui(input(time), |ui| demo.show(ui));
        assert!(
            !output.shapes.is_empty(),
            "gallery {expected:?} rendered empty frame"
        );
        output.drop_without_applying_deltas();
        time += 0.05;
        current = current.next();
    }
    assert_eq!(current, ActiveGallery::Default2D);
}

#[test]
fn default_theme_is_host_and_global_controls_sync_and_scroll_zoom_everywhere() {
    use crate::{appearance::ThemeChoice, gallery::ActiveGallery};

    let mut demo = Demo::default();
    assert_eq!(demo.options.appearance.theme, ThemeChoice::Host);
    assert!(demo.global_controls.open);
    assert!(demo.global_controls.scroll_zoom);

    // Wheel zoom changes state on Plot3D, Stage16, Multivariate(Stacked), Stage15(Bode+Smith), CustomAxes.
    for (gallery, target_cell) in [
        (ActiveGallery::Plot3D, 0),
        (ActiveGallery::Stage16, 0),
        (ActiveGallery::Multivariate, 0),
        (ActiveGallery::Stage15, 0),
        (ActiveGallery::Stage15, 2),
        (ActiveGallery::CustomAxes, 0),
    ] {
        let ctx = egui::Context::default();
        let mut d = Demo::default();
        if gallery == ActiveGallery::Multivariate {
            d.multivariate.view = crate::multivariate::MultivariateView::Stacked;
        }
        d.select_gallery(gallery);
        let mut rect = egui::Rect::NOTHING;
        ctx.run_ui(input(0.0), |ui| {
            let resps = d.show_charts(ui);
            rect = resps[target_cell].transform.rect();
        })
        .drop_without_applying_deltas();

        let center = rect.center();
        let mut wheel_in = input(0.1);
        wheel_in.events.push(egui::Event::PointerMoved(center));
        wheel_in.events.push(egui::Event::MouseWheel {
            unit: egui::MouseWheelUnit::Point,
            delta: egui::vec2(0.0, 50.0),
            phase: egui::TouchPhase::Move,
            source: egui::MouseWheelSource::Unknown,
            modifiers: egui::Modifiers::NONE,
        });
        let mut changed = false;
        ctx.run_ui(wheel_in, |ui| {
            let resps = d.show_charts(ui);
            changed = resps[target_cell].view_changed;
        })
        .drop_without_applying_deltas();
        assert!(
            changed,
            "expected wheel zoom to change view on {gallery:?} cell {target_cell}"
        );
        if gallery == ActiveGallery::CustomAxes {
            assert!(d.custom_axes.state_bounds(0).is_some());
        }
    }

    // Multipass & mode switches across all 4 Custom* galleries + reset button.
    let ctx = egui::Context::default();
    demo.custom_axes.fmt_mode = 3;
    demo.custom_axes.clamp_pan = false;
    demo.custom_annotations.upper_limit = 1.4;
    demo.custom_animation.speed_mult = 2.0;
    demo.custom_styling.log_colorbar = true;
    demo.custom_styling.columns = 4;
    for g in [
        ActiveGallery::CustomAxes,
        ActiveGallery::CustomAnnotations,
        ActiveGallery::CustomAnimation,
        ActiveGallery::CustomStyling,
    ] {
        demo.select_gallery(g);
        let mut passes = 0;
        let out = ctx.run_ui(input(0.2), |ui| {
            demo.show(ui);
            if passes == 0 {
                ui.ctx().request_discard("custom gallery multipass");
            }
            passes += 1;
        });
        assert_eq!(passes, 2);
        assert!(!out.shapes.is_empty());
        out.drop_without_applying_deltas();
        demo.reset_active_gallery();
    }
}

#[test]
fn os_theme_parser_and_pass_sync_fix_inverted_winit_theme_without_overriding_manual_choice() {
    use crate::os_theme::{parse_apps_use_light_theme, sync_pass_theme};

    let sample_dark = "\r\nHKEY_CURRENT_USER\\Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize\r\n    AppsUseLightTheme    REG_DWORD    0x0\r\n";
    let sample_light = "HKEY_CURRENT_USER\\Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize\n    appsuselighttheme    REG_DWORD    0x1\n";
    assert_eq!(
        parse_apps_use_light_theme(sample_dark),
        Some(egui::Theme::Dark)
    );
    assert_eq!(
        parse_apps_use_light_theme(sample_light),
        Some(egui::Theme::Light)
    );
    assert_eq!(
        parse_apps_use_light_theme("AppsUseLightTheme REG_DWORD 0x00000001"),
        Some(egui::Theme::Light)
    );
    assert_eq!(
        parse_apps_use_light_theme("AppsUseLightTheme REG_DWORD 0"),
        Some(egui::Theme::Dark)
    );
    assert_eq!(
        parse_apps_use_light_theme("AppsUseLightTheme REG_DWORD 1"),
        Some(egui::Theme::Light)
    );
    assert_eq!(parse_apps_use_light_theme("ERROR: value not found"), None);

    let ctx = egui::Context::default();
    let run_sync = |time: f64,
                    winit_system: Option<egui::Theme>,
                    true_os: Option<egui::Theme>,
                    force: bool| {
        let mut raw = input(time);
        raw.system_theme = winit_system;
        let mut visuals_dark = false;
        let out = ctx.run_ui(raw, |ui| {
            sync_pass_theme(ui, true_os, force);
            visuals_dark = ui.visuals().dark_mode;
        });
        let set_themes: Vec<egui::SystemTheme> = out
            .viewport_output
            .get(&egui::ViewportId::ROOT)
            .map(|v| {
                v.commands
                    .iter()
                    .filter_map(|c| match c {
                        egui::ViewportCommand::SetTheme(t) => Some(*t),
                        _ => None,
                    })
                    .collect()
            })
            .unwrap_or_default();
        out.drop_without_applying_deltas();
        (visuals_dark, set_themes)
    };

    // 1. OS is Light (AppsUseLightTheme=1), while winit reports stale/inverted Dark:
    let (dark, cmds) = run_sync(
        0.0,
        Some(egui::Theme::Dark),
        Some(egui::Theme::Light),
        false,
    );
    assert_eq!(ctx.theme(), egui::Theme::Light);
    assert!(!dark, "ui visuals should immediately switch to light");
    assert_eq!(cmds.last().copied(), Some(egui::SystemTheme::Light));

    // 2. OS switches to Dark (AppsUseLightTheme=0), while winit reports stale/inverted Light:
    let (dark, cmds) = run_sync(0.1, Some(egui::Theme::Light), Some(egui::Theme::Dark), true);
    assert_eq!(ctx.theme(), egui::Theme::Dark);
    assert!(dark, "ui visuals should immediately switch to dark");
    assert_eq!(cmds.last().copied(), Some(egui::SystemTheme::Dark));

    // 3. Explicit manual Light override while OS is Dark must be respected:
    ctx.set_theme(egui::ThemePreference::Light);
    let (dark, cmds) = run_sync(
        0.2,
        Some(egui::Theme::Light),
        Some(egui::Theme::Dark),
        false,
    );
    assert_eq!(ctx.theme(), egui::Theme::Light);
    assert!(!dark);
    assert_eq!(cmds.last().copied(), Some(egui::SystemTheme::Light));

    // 4. Switching back to System immediately tracks true OS Dark even when winit says Light:
    ctx.set_theme(egui::ThemePreference::System);
    let (dark, cmds) = run_sync(
        0.3,
        Some(egui::Theme::Light),
        Some(egui::Theme::Dark),
        false,
    );
    assert_eq!(ctx.theme(), egui::Theme::Dark);
    assert!(dark);
    assert_eq!(cmds.last().copied(), Some(egui::SystemTheme::Dark));
}

#[test]
fn stage16_40k_pointcloud_repeated_wheel_zoom_in_completes_quickly() {
    use crate::gallery::ActiveGallery;

    let ctx = egui::Context::default();
    let mut demo = Demo::default();
    demo.select_gallery(ActiveGallery::Stage16);

    let mut pc_rect = egui::Rect::NOTHING;
    let init_out = ctx.run_ui(input(0.0), |ui| {
        let resps = demo.show_charts(ui);
        pc_rect = resps[1].transform.rect();
    });
    assert!(!init_out.shapes.is_empty());
    init_out.drop_without_applying_deltas();

    let center = pc_rect.center();
    let mut zoom_steps = 0usize;
    for step in 1..=18 {
        let mut wheel_in = input(step as f64 * 0.05);
        wheel_in.events.push(egui::Event::PointerMoved(center));
        wheel_in.events.push(egui::Event::MouseWheel {
            unit: egui::MouseWheelUnit::Point,
            delta: egui::vec2(0.0, 120.0),
            phase: egui::TouchPhase::Move,
            source: egui::MouseWheelSource::Unknown,
            modifiers: egui::Modifiers::NONE,
        });
        let out = ctx.run_ui(wheel_in, |ui| {
            let resps = demo.show_charts(ui);
            if resps[1].view_changed {
                zoom_steps += 1;
            }
        });
        assert!(!out.shapes.is_empty());
        out.drop_without_applying_deltas();
    }
    assert!(
        zoom_steps > 0,
        "expected wheel zoom to advance 40k point cloud camera"
    );
}
