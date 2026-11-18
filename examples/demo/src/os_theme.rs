use eframe::egui;

/// Parses `reg query ... /v AppsUseLightTheme` stdout (`0x0` -> Dark, non-zero -> Light).
#[cfg(any(target_os = "windows", test))]
pub(crate) fn parse_apps_use_light_theme(stdout: &str) -> Option<egui::Theme> {
    for line in stdout.lines() {
        if !line.to_ascii_lowercase().contains("appsuselighttheme") {
            continue;
        }
        let token = line.split_whitespace().last()?;
        let val = if let Some(hex) = token
            .strip_prefix("0x")
            .or_else(|| token.strip_prefix("0X"))
        {
            u32::from_str_radix(hex, 16).ok()?
        } else {
            token.parse::<u32>().ok()?
        };
        return Some(if val == 0 {
            egui::Theme::Dark
        } else {
            egui::Theme::Light
        });
    }
    None
}

/// Applies the true OS theme (if supplied) for this pass and keeps native window theme in sync.
pub(crate) fn sync_pass_theme(
    ui: &mut egui::Ui,
    true_os_theme: Option<egui::Theme>,
    force_window_sync: bool,
) {
    ui.ctx().options_mut(|opt| {
        opt.sync_window_theme = false;
        if let Some(t) = true_os_theme {
            opt.fallback_theme = t;
            let patch = egui::RawInput {
                system_theme: Some(t),
                ..Default::default()
            };
            opt.begin_pass(&patch);
        }
    });

    let effective = ui.ctx().theme();
    let want_dark = effective == egui::Theme::Dark;
    if ui.style().visuals.dark_mode != want_dark {
        *ui.style_mut() = (*ui.ctx().global_style()).clone();
        ui.ctx().request_repaint();
    }

    let desired = match effective {
        egui::Theme::Dark => egui::SystemTheme::Dark,
        egui::Theme::Light => egui::SystemTheme::Light,
    };
    let id = egui::Id::NULL.with("customplot_win_theme_sync");
    let prev = ui.ctx().data(|d| d.get_temp::<egui::SystemTheme>(id));
    if prev != Some(desired) || force_window_sync {
        ui.ctx().data_mut(|d| d.insert_temp(id, desired));
        ui.ctx()
            .send_viewport_cmd(egui::ViewportCommand::SetTheme(desired));
    }
}

#[cfg(target_os = "windows")]
mod platform {
    use super::*;
    use std::os::windows::process::CommandExt;
    use std::path::PathBuf;
    use std::process::Command;
    use std::sync::{
        Arc, Condvar, Mutex,
        atomic::{AtomicU8, AtomicU64, Ordering},
    };
    use std::thread::JoinHandle;
    use std::time::Duration;

    const CREATE_NO_WINDOW: u32 = 0x0800_0000;

    #[derive(Default)]
    struct Ctrl {
        stop: bool,
        wake: bool,
    }

    struct Inner {
        latest: AtomicU8,
        generation: AtomicU64,
        ctrl: Mutex<Ctrl>,
        cv: Condvar,
        ctx: egui::Context,
    }

    pub(crate) struct OsThemeSync {
        inner: Arc<Inner>,
        thread: Option<JoinHandle<()>>,
        last_generation: u64,
        last_winit_theme: Option<egui::Theme>,
    }

    const fn encode_theme(theme: Option<egui::Theme>) -> u8 {
        match theme {
            None => 0,
            Some(egui::Theme::Dark) => 1,
            Some(egui::Theme::Light) => 2,
        }
    }

    const fn decode_theme(code: u8) -> Option<egui::Theme> {
        match code {
            1 => Some(egui::Theme::Dark),
            2 => Some(egui::Theme::Light),
            _ => None,
        }
    }

    fn reg_exe_path() -> PathBuf {
        std::env::var_os("SystemRoot")
            .map(|r| PathBuf::from(r).join("System32").join("reg.exe"))
            .unwrap_or_else(|| PathBuf::from("reg.exe"))
    }

    fn query_windows_app_theme() -> Option<egui::Theme> {
        let output = Command::new(reg_exe_path())
            .args([
                "query",
                r"HKCU\Software\Microsoft\Windows\CurrentVersion\Themes\Personalize",
                "/v",
                "AppsUseLightTheme",
            ])
            .creation_flags(CREATE_NO_WINDOW)
            .output()
            .ok()?;
        if !output.status.success() {
            return None;
        }
        parse_apps_use_light_theme(&String::from_utf8_lossy(&output.stdout))
    }

    fn wait_stop(inner: &Inner, dur: Duration) -> bool {
        let ctrl = inner
            .ctrl
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let (guard, _) = inner
            .cv
            .wait_timeout_while(ctrl, dur, |c| !c.stop)
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        guard.stop
    }

    fn background_loop(inner: Arc<Inner>) {
        let mut last_sent = decode_theme(inner.latest.load(Ordering::Acquire));
        loop {
            let woke = {
                let ctrl = inner
                    .ctrl
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                if ctrl.stop {
                    return;
                }
                let (mut guard, _) = inner
                    .cv
                    .wait_timeout_while(ctrl, Duration::from_millis(600), |c| !c.stop && !c.wake)
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                if guard.stop {
                    return;
                }
                std::mem::take(&mut guard.wake)
            };

            let mut poll_and_notify = || {
                let observed = query_windows_app_theme();
                if observed.is_some() && observed != last_sent {
                    last_sent = observed;
                    inner
                        .latest
                        .store(encode_theme(observed), Ordering::Release);
                    inner.generation.fetch_add(1, Ordering::AcqRel);
                    inner.ctx.request_repaint();
                }
            };

            if woke {
                if wait_stop(&inner, Duration::from_millis(25)) {
                    return;
                }
                poll_and_notify();
                if wait_stop(&inner, Duration::from_millis(140)) {
                    return;
                }
                poll_and_notify();
            } else {
                poll_and_notify();
            }
        }
    }

    impl OsThemeSync {
        pub(crate) fn install(ctx: &egui::Context) -> Self {
            let initial = query_windows_app_theme();
            ctx.options_mut(|opt| {
                opt.sync_window_theme = false;
                if let Some(t) = initial {
                    opt.fallback_theme = t;
                    let patch = egui::RawInput {
                        system_theme: Some(t),
                        ..Default::default()
                    };
                    opt.begin_pass(&patch);
                }
            });
            let effective = ctx.theme();
            let desired = match effective {
                egui::Theme::Dark => egui::SystemTheme::Dark,
                egui::Theme::Light => egui::SystemTheme::Light,
            };
            ctx.data_mut(|d| {
                d.insert_temp(egui::Id::NULL.with("customplot_win_theme_sync"), desired);
            });
            ctx.send_viewport_cmd(egui::ViewportCommand::SetTheme(desired));

            let inner = Arc::new(Inner {
                latest: AtomicU8::new(encode_theme(initial)),
                generation: AtomicU64::new(0),
                ctrl: Mutex::new(Ctrl::default()),
                cv: Condvar::new(),
                ctx: ctx.clone(),
            });
            let worker = Arc::clone(&inner);
            let thread = std::thread::Builder::new()
                .name("customplot-os-theme".into())
                .spawn(move || background_loop(worker))
                .ok();
            Self {
                inner,
                thread,
                last_generation: 0,
                last_winit_theme: None,
            }
        }

        pub(crate) fn current_theme(&self) -> Option<egui::Theme> {
            decode_theme(self.inner.latest.load(Ordering::Acquire))
        }

        fn request_recheck(&self) {
            {
                let mut ctrl = self
                    .inner
                    .ctrl
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                ctrl.wake = true;
            }
            self.inner.cv.notify_all();
        }

        pub(crate) fn sync_ui(&mut self, ui: &mut egui::Ui) {
            let raw_winit_theme = ui.input(|i| i.raw.system_theme);
            let focused = ui.input(|i| {
                i.events
                    .iter()
                    .any(|e| matches!(e, egui::Event::WindowFocused(true)))
            });
            let mut force_window_sync = false;
            if raw_winit_theme != self.last_winit_theme || focused {
                self.last_winit_theme = raw_winit_theme;
                self.request_recheck();
                force_window_sync = true;
            }
            let generation = self.inner.generation.load(Ordering::Acquire);
            if generation != self.last_generation {
                self.last_generation = generation;
                force_window_sync = true;
            }
            sync_pass_theme(ui, self.current_theme(), force_window_sync);
        }
    }

    impl Drop for OsThemeSync {
        fn drop(&mut self) {
            {
                let mut ctrl = self
                    .inner
                    .ctrl
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                ctrl.stop = true;
            }
            self.inner.cv.notify_all();
            if let Some(handle) = self.thread.take() {
                let _ = handle.join();
            }
        }
    }
}

#[cfg(not(target_os = "windows"))]
mod platform {
    use super::*;

    pub(crate) struct OsThemeSync;

    impl OsThemeSync {
        pub(crate) fn install(_ctx: &egui::Context) -> Self {
            Self
        }

        pub(crate) fn current_theme(&self) -> Option<egui::Theme> {
            None
        }

        pub(crate) fn sync_ui(&mut self, ui: &mut egui::Ui) {
            sync_pass_theme(ui, None, false);
        }
    }
}

pub(crate) use platform::OsThemeSync;
