use eframe::egui;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum Language {
    #[default]
    English,
    ZhCn,
    Japanese,
}

impl Language {
    pub const ALL: [Self; 3] = [Self::ZhCn, Self::English, Self::Japanese];

    pub const fn index(self) -> usize {
        match self {
            Self::English => 0,
            Self::ZhCn => 1,
            Self::Japanese => 2,
        }
    }

    pub const fn label(self) -> &'static str {
        match self {
            Self::English => "English",
            Self::ZhCn => "简体中文",
            Self::Japanese => "日本語",
        }
    }

    /// Always begins with `"egui-customplot"` so window smoke tools match reliably.
    pub const fn window_title(self) -> &'static str {
        match self {
            Self::English => "egui-customplot | Interactive Demo",
            Self::ZhCn => "egui-customplot | 交互演示",
            Self::Japanese => "egui-customplot | インタラクティブデモ",
        }
    }

    pub fn from_token(raw: &str) -> Option<Self> {
        let lower = raw.trim().to_ascii_lowercase();
        if lower == "en" || lower.starts_with("en_") || lower.starts_with("en-") {
            Some(Self::English)
        } else if lower == "zh"
            || lower == "cn"
            || lower.starts_with("zh_")
            || lower.starts_with("zh-")
        {
            Some(Self::ZhCn)
        } else if lower == "ja"
            || lower == "jp"
            || lower.starts_with("ja_")
            || lower.starts_with("ja-")
        {
            Some(Self::Japanese)
        } else {
            None
        }
    }

    pub fn from_args(args: &[String]) -> Option<Self> {
        args.iter().find_map(|arg| {
            if let Some(val) = arg.strip_prefix("--lang=") {
                Self::from_token(val)
            } else {
                match arg.as_str() {
                    "--zh" | "--zh-cn" => Some(Self::ZhCn),
                    "--en" => Some(Self::English),
                    "--ja" => Some(Self::Japanese),
                    _ => None,
                }
            }
        })
    }

    pub fn detect_default(args: &[String]) -> Self {
        if let Some(explicit) = Self::from_args(args) {
            return explicit;
        }
        for key in ["CUSTOMPLOT_LANG", "LC_ALL", "LANG"] {
            if let Ok(val) = std::env::var(key)
                && let Some(parsed) = Self::from_token(&val)
            {
                return parsed;
            }
        }
        Self::ZhCn
    }
}

pub(crate) fn is_lang_arg(arg: &str) -> bool {
    arg.starts_with("--lang=") || matches!(arg, "--zh" | "--zh-cn" | "--en" | "--ja")
}

#[inline]
pub(crate) fn tr<'a>(l: Language, en: &'a str, zh: &'a str, ja: &'a str) -> &'a str {
    match l {
        Language::English => en,
        Language::ZhCn => zh,
        Language::Japanese => ja,
    }
}

fn context_key() -> egui::Id {
    egui::Id::NULL
}

pub(crate) fn sync_context(ctx: &egui::Context, lang: Language) {
    if lang != Language::English {
        crate::fonts::ensure_cjk_fonts(ctx);
    }
    let prev = ctx.data(|d| d.get_temp::<Language>(context_key()));
    if prev != Some(lang) {
        ctx.data_mut(|d| d.insert_temp(context_key(), lang));
        if prev.is_some() {
            ctx.send_viewport_cmd(egui::ViewportCommand::Title(lang.window_title().to_owned()));
        }
    }
}

pub(crate) fn lang(ctx: &egui::Context) -> Language {
    ctx.data(|d| d.get_temp::<Language>(context_key()))
        .unwrap_or(Language::English)
}

pub(crate) fn language_selector(ui: &mut egui::Ui, lang: &mut Language) {
    ui.horizontal(|ui| {
        ui.label(tr(*lang, "Lang:", "语言:", "言語:"));
        for candidate in Language::ALL {
            ui.selectable_value(lang, candidate, candidate.label());
        }
    });
}

fn paint_theme_icon(
    painter: &egui::Painter,
    center: egui::Pos2,
    pref: egui::ThemePreference,
    fg: egui::Color32,
) {
    match pref {
        egui::ThemePreference::Light => {
            painter.circle_filled(center, 2.7, fg);
            for k in 0..8 {
                let ang = k as f32 * std::f32::consts::FRAC_PI_4;
                let (s, c) = ang.sin_cos();
                painter.line_segment(
                    [
                        egui::pos2(center.x + c * 4.1, center.y + s * 4.1),
                        egui::pos2(center.x + c * 6.0, center.y + s * 6.0),
                    ],
                    egui::Stroke::new(1.3, fg),
                );
            }
        }
        egui::ThemePreference::Dark => {
            // Vector crescent built as a triangle strip between outer and inset arcs.
            let n = 14usize;
            let mut mesh = egui::Mesh::default();
            let a0 = std::f32::consts::PI * 0.25;
            let a1 = std::f32::consts::PI * 1.75;
            let c_in = egui::pos2(center.x + 2.3, center.y - 1.1);
            for i in 0..=n {
                let t = i as f32 / n as f32;
                let a = a0 + (a1 - a0) * t;
                let p_out = egui::pos2(center.x + a.cos() * 5.6, center.y + a.sin() * 5.6);
                let p_in = egui::pos2(c_in.x + a.cos() * 4.3, c_in.y + a.sin() * 4.3);
                let base = mesh.vertices.len() as u32;
                mesh.colored_vertex(p_out, fg);
                mesh.colored_vertex(p_in, fg);
                if i > 0 {
                    mesh.add_triangle(base - 2, base - 1, base);
                    mesh.add_triangle(base - 1, base + 1, base);
                }
            }
            painter.add(egui::Shape::mesh(mesh));
        }
        egui::ThemePreference::System => {
            painter.circle_stroke(center, 5.2, egui::Stroke::new(1.3, fg));
            let n = 12usize;
            let mut mesh = egui::Mesh::default();
            mesh.colored_vertex(center, fg);
            let a0 = std::f32::consts::FRAC_PI_2;
            let a1 = std::f32::consts::PI * 1.5;
            for i in 0..=n {
                let a = a0 + (a1 - a0) * (i as f32 / n as f32);
                mesh.colored_vertex(
                    egui::pos2(center.x + a.cos() * 5.0, center.y + a.sin() * 5.0),
                    fg,
                );
                if i > 0 {
                    let b = mesh.vertices.len() as u32;
                    mesh.add_triangle(0, b - 2, b - 1);
                }
            }
            painter.add(egui::Shape::mesh(mesh));
        }
    }
}

fn theme_icon_button(
    ui: &mut egui::Ui,
    pref: egui::ThemePreference,
    selected: bool,
    label: &str,
    tooltip: &str,
) -> egui::Response {
    let font_id = egui::TextStyle::Button.resolve(ui.style());
    let galley = ui
        .painter()
        .layout_no_wrap(label.to_owned(), font_id, egui::Color32::PLACEHOLDER);
    let pad = ui.spacing().button_padding;
    let icon_w = 14.0_f32;
    let gap = 4.0_f32;
    let size = egui::vec2(
        pad.x * 2.0 + icon_w + gap + galley.size().x,
        galley.size().y.max(icon_w) + pad.y * 2.0,
    );
    let (rect, response) = ui.allocate_exact_size(size, egui::Sense::click());
    if ui.is_rect_visible(rect) {
        let visuals = ui.style().interact_selectable(&response, selected);
        let painter = ui.painter();
        painter.rect_filled(rect, visuals.corner_radius, visuals.weak_bg_fill);
        painter.rect_stroke(
            rect,
            visuals.corner_radius,
            visuals.bg_stroke,
            egui::StrokeKind::Inside,
        );
        let icon_center = egui::pos2(rect.left() + pad.x + icon_w * 0.5, rect.center().y);
        paint_theme_icon(painter, icon_center, pref, visuals.fg_stroke.color);
        let text_pos = egui::pos2(
            rect.left() + pad.x + icon_w + gap,
            rect.center().y - galley.size().y * 0.5,
        );
        painter.galley(text_pos, galley, visuals.text_color());
    }
    response.on_hover_text(tooltip)
}

pub(crate) fn theme_preference_selector(ui: &mut egui::Ui, l: Language) {
    let current = ui.ctx().options(|o| o.theme_preference);
    ui.horizontal(|ui| {
        ui.label(tr(l, "Theme:", "外观:", "外観:"));
        for (pref, label, tip) in [
            (
                egui::ThemePreference::Light,
                tr(l, "Light", "浅色", "ライト"),
                tr(
                    l,
                    "Switch to light mode",
                    "切换为浅色主题",
                    "ライトテーマへ切替",
                ),
            ),
            (
                egui::ThemePreference::Dark,
                tr(l, "Dark", "深色", "ダーク"),
                tr(
                    l,
                    "Switch to dark mode",
                    "切换为深色主题",
                    "ダークテーマへ切替",
                ),
            ),
            (
                egui::ThemePreference::System,
                tr(l, "Auto", "跟随", "自動"),
                tr(
                    l,
                    "Follow system light/dark preference",
                    "跟随系统亮暗主题设置",
                    "システムのライト/ダーク設定に従う",
                ),
            ),
        ] {
            if theme_icon_button(ui, pref, current == pref, label, tip).clicked() {
                ui.ctx().set_theme(pref);
            }
        }
    });
}
