use std::time::Instant;

use eframe::egui;
use egui_customplot::{LineSampling, PlotCache};

use crate::data::DemoData;

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
            sampling: LineSampling::Exact,
            indexed: false,
            cached: false,
            streaming: false,
            index_budget_mib: 16,
            cache_budget_mib: 8,
        }
    }
}

impl Options {
    pub fn show(&mut self, ui: &mut egui::Ui) -> bool {
        let mut reindex = false;
        ui.collapsing("Data performance / budgets", |ui| {
            reindex |= ui.checkbox(&mut self.indexed, "Build monotonic-X indexes").changed();
            egui::ComboBox::from_id_salt("sampling").selected_text(format!("{:?}", self.sampling))
                .show_ui(ui, |ui| {
                    for sampling in [LineSampling::Exact, LineSampling::Visible, LineSampling::MinMax] {
                        ui.selectable_value(&mut self.sampling, sampling, format!("{sampling:?}"));
                    }
                });
            ui.small("Visible/MinMax fall back to Exact without an index. MinMax is approximate; markers stay raw.");
            ui.add(egui::Slider::new(&mut self.index_budget_mib, 0..=64).text("MiB / index"));
            reindex |= ui.button("Apply index budget / retry").clicked();
            ui.checkbox(&mut self.cached, "Host-owned line geometry cache");
            ui.add(egui::Slider::new(&mut self.cache_budget_mib, 0..=32).text("MiB / cache"));
            ui.checkbox(&mut self.streaming, "Live signal: bounded 65,536-point ring");
            ui.small("Live mode replaces static series. 256 samples / tick; oldest overwritten. Snapshot + optional index rebuilt at 30 Hz, not every layout pass.");
        });
        reindex
    }
}

pub(super) struct Performance {
    pub cache: PlotCache,
    pub live: crate::stream::LiveSignal,
    index_status: String,
}

impl Default for Performance {
    fn default() -> Self {
        Self {
            cache: PlotCache::default(),
            live: crate::stream::LiveSignal::default(),
            index_status:
                "Indexes not built; preprocessing is explicit and synchronous in this demo.".into(),
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
            format!(
                "Index request: {:.3} ms. Budgets exclude samples and rendering scratch.",
                start.elapsed().as_secs_f64() * 1000.0
            )
        } else {
            errors.join("\n")
        };
    }

    pub fn status(&self, ui: &mut egui::Ui) {
        ui.small(&self.index_status);
        ui.small(format!(
            "Host cache: {} entries / {} bytes",
            self.cache.entry_count(),
            self.cache.retained_bytes()
        ));
        if let Some(error) = &self.live.error {
            ui.colored_label(egui::Color32::YELLOW, error);
        }
    }
}
