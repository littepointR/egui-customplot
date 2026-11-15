use std::time::Duration;

use eframe::egui;
use egui_customplot::{PlotData, PlotStream};

use crate::data::Scene;

pub(super) struct LiveSignal {
    ring: PlotStream,
    pub data: PlotData,
    pub error: Option<String>,
    next_sample: u64,
    last_time: Option<f64>,
}

impl Default for LiveSignal {
    fn default() -> Self {
        Self {
            ring: PlotStream::new(65_536, 65_536 * 16).expect("fixed demo budget"),
            data: PlotData::new(Vec::new()),
            error: None,
            next_sample: 0,
            last_time: None,
        }
    }
}

impl LiveSignal {
    pub fn tick(
        &mut self,
        ctx: &egui::Context,
        scene: Scene,
        gaps: bool,
        index_budget: Option<usize>,
    ) {
        let now = ctx.input(|i| i.time);
        if self.last_time.is_none_or(|t| now - t >= 1.0 / 30.0) {
            let batch = std::array::from_fn::<_, 256, _>(|i| {
                let sample = self.next_sample + i as u64;
                let x = (sample + 1) as f64 / 512.0;
                let y = if gaps && (2000..2080).contains(&(sample % 4096)) {
                    f64::NAN
                } else {
                    match scene {
                        Scene::Signal => x.sin(),
                        Scene::PowerLaw => x.powf(0.65),
                    }
                };
                [x, y]
            });
            let report = self
                .ring
                .append_once(ctx.cumulative_frame_nr(), &batch)
                .expect("monotonic demo frames");
            if report.applied {
                self.next_sample += 256;
                self.data = self.ring.snapshot();
                self.error = index_budget
                    .and_then(|budget| self.data.build_line_index(budget).err())
                    .map(|e| e.to_string());
                self.last_time = Some(now);
            }
        }
        ctx.request_repaint_after(Duration::from_secs_f64(1.0 / 30.0));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn live_demo_publishes_once_per_tick_even_with_discarded_layout() {
        let ctx = egui::Context::default();
        let mut live = LiveSignal::default();
        let mut passes = 0;
        let mut version = None;
        ctx.run_ui(egui::RawInput::default(), |ui| {
            live.tick(ui.ctx(), Scene::Signal, true, Some(1024 * 1024));
            assert_eq!(live.data.len(), 256);
            if let Some(version) = version {
                assert_eq!(live.data.version(), version);
            }
            version = Some(live.data.version());
            if passes == 0 {
                ui.ctx().request_discard("stream layout test");
            }
            passes += 1;
        })
        .drop_without_applying_deltas();
        assert_eq!(passes, 2);
        assert_eq!(live.ring.revision(), 1);
        assert!(live.data.line_index_bytes() > 0);
    }
}
