use std::time::Instant;

use eframe::egui;
use egui_customplot::{LineSampling, PlotCache};

use crate::{
    data::DemoData,
    i18n::{Language, lang, tr},
};

pub(super) struct Options {
    pub sampling: LineSampling,
    pub indexed: bool,
    pub cached: bool,
    pub streaming: bool,
    pub index_budget_mib: usize,
    pub cache_budget_mib: usize,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            sampling: LineSampling::Auto,
            indexed: true,
            cached: true,
            streaming: false,
            index_budget_mib: 64,
            cache_budget_mib: 8,
        }
    }
}

impl Options {
    pub fn show(&mut self, ui: &mut egui::Ui) -> bool {
        let l = lang(ui.ctx());
        let mut reindex = false;
        egui::CollapsingHeader::new(tr(
            l,
            "Data performance / budgets",
            "数据性能 / 内存预算",
            "データ性能 / メモリ予算",
        ))
        .id_salt("perf_budgets_collapsing")
        .show(ui, |ui| {
            reindex |= ui
                .checkbox(
                    &mut self.indexed,
                    tr(
                        l,
                        "Build monotonic-X indexes",
                        "构建单调 X 树索引",
                        "単調 X ツリー索引を構築",
                    ),
                )
                .changed();
            let sampling_label = |s: LineSampling| match s {
                LineSampling::Auto => tr(l, "Auto", "Auto（自动 LOD）", "Auto（自動 LOD）"),
                LineSampling::Exact => tr(l, "Exact", "Exact（精确全量）", "Exact（厳密）"),
                LineSampling::Visible => {
                    tr(l, "Visible", "Visible（视区裁剪）", "Visible（表示範囲）")
                }
                LineSampling::MinMax => tr(l, "MinMax", "MinMax（像素极值）", "MinMax（最大最小）"),
            };
            egui::ComboBox::from_id_salt("sampling")
                .selected_text(sampling_label(self.sampling))
                .show_ui(ui, |ui| {
                    for sampling in [
                        LineSampling::Auto,
                        LineSampling::Exact,
                        LineSampling::Visible,
                        LineSampling::MinMax,
                    ] {
                        ui.selectable_value(&mut self.sampling, sampling, sampling_label(sampling));
                    }
                });
            ui.small(tr(
                l,
                "Visible/MinMax fall back to Exact without an index. MinMax is approximate; markers stay raw.",
                "无索引时 Visible/MinMax 回退为 Exact；Auto 自动启用 LOD。",
                "索引なしでは Exact にフォールバック；Auto は自動 LOD を使用します。",
            ));
            ui.add(
                egui::Slider::new(&mut self.index_budget_mib, 0..=64)
                    .text(tr(l, "MiB / index", "MiB / 索引", "MiB / 索引")),
            );
            reindex |= ui
                .button(tr(
                    l,
                    "Apply index budget / retry",
                    "应用索引预算 / 重建",
                    "索引予算を適用 / 再構築",
                ))
                .clicked();
            ui.checkbox(
                &mut self.cached,
                tr(
                    l,
                    "Host-owned line geometry cache",
                    "宿主有界折线几何缓存",
                    "ホスト管理ラインキャッシュ",
                ),
            );
            ui.add(
                egui::Slider::new(&mut self.cache_budget_mib, 0..=32)
                    .text(tr(l, "MiB / cache", "MiB / 缓存", "MiB / キャッシュ")),
            );
            ui.checkbox(
                &mut self.streaming,
                tr(
                    l,
                    "Live signal: bounded 65,536-point ring",
                    "实时流信号：65,536 点有界环形缓冲",
                    "ライブ信号：65,536 点リング",
                ),
            );
            ui.small(tr(
                l,
                "Live mode replaces static series. 256 samples / tick; oldest overwritten. Snapshot + optional index rebuilt at 30 Hz, not every layout pass.",
                "实时模式以 30 Hz 每次追加 256 点并覆盖最旧样本，多次布局仅发布一次。",
                "ライブモードは 30 Hz で 256 点ずつ追記し、複数パスでも重複更新しません。",
            ));
        });
        reindex
    }
}

enum IndexStatus {
    NotBuilt,
    Built { ms: f64 },
    Failed(String),
}

pub(super) struct Performance {
    pub cache: PlotCache,
    pub live: crate::stream::LiveSignal,
    index_status: IndexStatus,
}

impl Default for Performance {
    fn default() -> Self {
        Self {
            cache: PlotCache::default(),
            live: crate::stream::LiveSignal::default(),
            index_status: IndexStatus::NotBuilt,
        }
    }
}

impl Performance {
    pub fn index(&mut self, data: &mut DemoData, options: &Options) {
        let start = Instant::now();
        let mut errors = Vec::new();
        for (name, samples) in [
            ("signal", &mut data.signal),
            ("reference", &mut data.reference),
            ("pressure", &mut data.secondary),
            ("scatter", &mut data.observations),
        ] {
            if options.indexed {
                if let Err(error) = samples.build_line_index(options.index_budget_mib * 1_048_576) {
                    errors.push(format!("{name}: {error}"));
                }
            } else {
                samples.clear_line_index();
            }
        }
        self.index_status = if errors.is_empty() {
            IndexStatus::Built {
                ms: start.elapsed().as_secs_f64() * 1000.0,
            }
        } else {
            IndexStatus::Failed(errors.join("\n"))
        };
    }

    pub fn status(&self, ui: &mut egui::Ui) {
        let l = lang(ui.ctx());
        match &self.index_status {
            IndexStatus::NotBuilt => {
                ui.small(tr(
                    l,
                    "Indexes ready (PlotData::new auto-indexes monotonic series >= 8,192 pts).",
                    "索引就绪（PlotData::new 会对 ≥ 8,192 点的单调 X 数据自动建树）。",
                    "索引準備完了（8,192 点以上の単調 X データは自動構築されます）。",
                ));
            }
            IndexStatus::Built { ms } => {
                let msg = match l {
                    Language::English => format!(
                        "Index request: {ms:.3} ms. Budgets exclude samples and rendering scratch."
                    ),
                    Language::ZhCn => {
                        format!("索引构建耗时：{ms:.3} ms（预算不含样本与临时草稿）。")
                    }
                    Language::Japanese => {
                        format!("索引処理時間：{ms:.3} ms（予算はデータ本体を除く）。")
                    }
                };
                ui.small(msg);
            }
            IndexStatus::Failed(err) => {
                ui.small(err);
            }
        }
        let cache_msg = match l {
            Language::English => format!(
                "Host cache: {} entries / {} bytes",
                self.cache.entry_count(),
                self.cache.retained_bytes()
            ),
            Language::ZhCn => format!(
                "宿主缓存：{} 条目 / {} 字节",
                self.cache.entry_count(),
                self.cache.retained_bytes()
            ),
            Language::Japanese => format!(
                "ホストキャッシュ：{} 件 / {} bytes",
                self.cache.entry_count(),
                self.cache.retained_bytes()
            ),
        };
        ui.small(cache_msg);
        if let Some(error) = &self.live.error {
            ui.colored_label(egui::Color32::YELLOW, error);
        }
    }
}
