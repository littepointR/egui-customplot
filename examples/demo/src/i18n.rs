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
            Self::English => "egui-customplot | Stage 11",
            Self::ZhCn => "egui-customplot | 阶段 11 演示",
            Self::Japanese => "egui-customplot | ステージ 11 デモ",
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
