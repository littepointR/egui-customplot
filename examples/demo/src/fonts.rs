use std::{fs, path::Path, sync::Arc};

use eframe::egui;

#[cfg(target_os = "windows")]
const PRIMARY_CJK_FONTS: &[&str] = &[
    r"C:\Windows\Fonts\msyh.ttc",
    r"C:\Windows\Fonts\Deng.ttf",
    r"C:\Windows\Fonts\NotoSansSC-VF.ttf",
    r"C:\Windows\Fonts\simhei.ttf",
    r"C:\Windows\Fonts\simsun.ttc",
];

#[cfg(target_os = "macos")]
const PRIMARY_CJK_FONTS: &[&str] = &[
    "/System/Library/Fonts/PingFang.ttc",
    "/System/Library/Fonts/Hiragino Sans GB.ttc",
    "/System/Library/Fonts/STHeiti Light.ttc",
];

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
const PRIMARY_CJK_FONTS: &[&str] = &[
    "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
    "/usr/share/fonts/noto-cjk/NotoSansCJK-Regular.ttc",
    "/usr/share/fonts/truetype/noto/NotoSansCJK-Regular.ttc",
    "/usr/share/fonts/truetype/wqy/wqy-microhei.ttc",
];

#[cfg(target_os = "windows")]
const SECONDARY_JA_FONTS: &[&str] = &[
    r"C:\Windows\Fonts\YuGothR.ttc",
    r"C:\Windows\Fonts\meiryo.ttc",
    r"C:\Windows\Fonts\SourceHanSansJP-Normal.otf",
];

#[cfg(target_os = "macos")]
const SECONDARY_JA_FONTS: &[&str] = &["/System/Library/Fonts/Hiragino Sans W3.ttc"];

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
const SECONDARY_JA_FONTS: &[&str] = &[];

#[derive(Clone, Copy)]
struct CjkFontsAttempted(bool);

fn first_readable_font(candidates: &[&'static str]) -> Option<(&'static str, Vec<u8>)> {
    candidates.iter().find_map(|&path| {
        Path::new(path)
            .is_file()
            .then(|| fs::read(path).ok().map(|bytes| (path, bytes)))
            .flatten()
    })
}

/// Loads system CJK fallback fonts into `ctx` at most once per `egui::Context`.
pub(crate) fn ensure_cjk_fonts(ctx: &egui::Context) -> bool {
    if let Some(attempted) = ctx.data(|d| d.get_temp::<CjkFontsAttempted>(egui::Id::NULL)) {
        return attempted.0;
    }

    let mut defs = egui::FontDefinitions::default();
    let mut loaded_keys = Vec::new();
    let mut primary_path = None;

    if let Some((path, bytes)) = first_readable_font(PRIMARY_CJK_FONTS) {
        let name = "customplot_cjk_primary".to_owned();
        defs.font_data
            .insert(name.clone(), Arc::new(egui::FontData::from_owned(bytes)));
        loaded_keys.push(name);
        primary_path = Some(path);
    }

    if let Some((path, bytes)) = first_readable_font(SECONDARY_JA_FONTS)
        && primary_path != Some(path)
    {
        let name = "customplot_cjk_ja".to_owned();
        defs.font_data
            .insert(name.clone(), Arc::new(egui::FontData::from_owned(bytes)));
        loaded_keys.push(name);
    }

    let installed = !loaded_keys.is_empty();
    if installed {
        for family in [egui::FontFamily::Proportional, egui::FontFamily::Monospace] {
            let entry = defs.families.entry(family).or_default();
            for name in &loaded_keys {
                entry.push(name.clone());
            }
        }
        ctx.set_fonts(defs);
    }
    ctx.data_mut(|d| d.insert_temp(egui::Id::NULL, CjkFontsAttempted(installed)));
    installed
}
