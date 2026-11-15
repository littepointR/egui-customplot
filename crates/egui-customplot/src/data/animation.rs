use std::time::Duration;

use egui::Context;

use crate::{AppendReport, PlotData, PlotPoint, PlotStream, StreamError};

/// Host-owned bounded history for `animatedline` (`ML-078`), updating snapshots
/// only on applied batches or clears so multi-pass `Plot::show` stays read-only.
pub struct AnimatedLine2D {
    stream: PlotStream,
    published: PlotData,
    paused: bool,
}

impl AnimatedLine2D {
    pub fn new(capacity_points: usize, budget_bytes: usize) -> Result<Self, StreamError> {
        let stream = PlotStream::new(capacity_points, budget_bytes)?;
        let published = stream.snapshot();
        Ok(Self {
            stream,
            published,
            paused: false,
        })
    }

    pub fn set_paused(&mut self, paused: bool) {
        self.paused = paused;
    }

    pub fn is_paused(&self) -> bool {
        self.paused
    }

    pub fn add_points_once(
        &mut self,
        batch_id: u64,
        points: &[PlotPoint],
    ) -> Result<AppendReport, StreamError> {
        if self.paused {
            return Ok(AppendReport {
                applied: false,
                revision: self.stream.revision(),
                retained_input_points: 0,
                evicted_points: 0,
                dropped_input_points: 0,
            });
        }
        let report = self.stream.append_once(batch_id, points)?;
        if report.applied && report.revision != self.published.version() {
            self.published = self.stream.snapshot();
        }
        Ok(report)
    }

    pub fn clear(&mut self) -> Result<(), StreamError> {
        let before = self.stream.revision();
        self.stream.clear()?;
        if self.stream.revision() != before {
            self.published = self.stream.snapshot();
        }
        Ok(())
    }

    pub fn data(&self) -> &PlotData {
        &self.published
    }

    pub fn revision(&self) -> u64 {
        self.stream.revision()
    }

    pub fn len(&self) -> usize {
        self.stream.len()
    }

    pub fn is_empty(&self) -> bool {
        self.stream.is_empty()
    }

    /// Requests a repaint only when active and unacknowledged revision changes exist (EGUI-08).
    pub fn sync_repaint(&self, ctx: &Context, last_seen_revision: &mut u64) -> bool {
        let rev = self.stream.revision();
        if !self.paused && rev != *last_seen_revision {
            *last_seen_revision = rev;
            ctx.request_repaint();
            true
        } else {
            false
        }
    }
}

/// Host-owned frame-idempotent playback state for `comet` (`ML-079`).
#[derive(Clone, Debug)]
pub struct CometPlayback {
    speed_points_per_sec: f64,
    tail_fraction: f64,
    looping: bool,
    playing: bool,
    progress_points: f64,
    last_frame_id: Option<u64>,
}

impl CometPlayback {
    pub fn new(speed_points_per_sec: f64) -> Self {
        Self {
            speed_points_per_sec: if speed_points_per_sec.is_finite() && speed_points_per_sec > 0.0
            {
                speed_points_per_sec
            } else {
                30.0
            },
            tail_fraction: 0.2,
            looping: false,
            playing: true,
            progress_points: 0.0,
            last_frame_id: None,
        }
    }

    pub fn tail_fraction(mut self, fraction: f64) -> Self {
        if fraction.is_finite() {
            self.tail_fraction = fraction.clamp(0.0, 1.0);
        }
        self
    }

    pub fn looping(mut self, looping: bool) -> Self {
        self.looping = looping;
        self
    }

    pub fn set_playing(&mut self, playing: bool) {
        self.playing = playing;
    }

    pub fn is_playing(&self) -> bool {
        self.playing
    }

    pub fn reset(&mut self) {
        self.progress_points = 0.0;
        self.last_frame_id = None;
    }

    pub fn set_progress_points(&mut self, progress: f64, total_points: usize) {
        let max_idx = total_points.saturating_sub(1) as f64;
        if progress.is_finite() {
            self.progress_points = progress.clamp(0.0, max_idx);
        }
    }

    pub fn progress_points(&self) -> f64 {
        self.progress_points
    }

    pub fn tail_fraction_value(&self) -> f64 {
        self.tail_fraction
    }

    /// Advances once per distinct host frame ID (`EGUI-05` multi-pass safe).
    pub fn advance_once(&mut self, frame_id: u64, dt_secs: f64, total_points: usize) -> bool {
        if !self.playing || total_points <= 1 || !dt_secs.is_finite() || dt_secs <= 0.0 {
            return false;
        }
        if self.last_frame_id == Some(frame_id) {
            return false;
        }
        self.last_frame_id = Some(frame_id);
        let max_idx = (total_points - 1) as f64;
        let next = self.progress_points + dt_secs * self.speed_points_per_sec;
        if next >= max_idx {
            if self.looping {
                self.progress_points = next % max_idx;
            } else {
                self.progress_points = max_idx;
                self.playing = false;
            }
        } else {
            self.progress_points = next;
        }
        true
    }

    /// Schedules timed redraw only while actively playing across `> 1` samples (`EGUI-08`).
    pub fn sync_repaint(&self, ctx: &Context, total_points: usize, interval: Duration) -> bool {
        if self.playing && total_points > 1 {
            ctx.request_repaint_after(interval);
            true
        } else {
            false
        }
    }
}
