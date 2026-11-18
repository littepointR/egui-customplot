use std::time::Duration;

use egui::Context;

use crate::{AxisScale, Point3D};

use super::Streamline3DData;

/// Host-owned seeded particle dataset along 3D streamlines (`streamparticles`, `ML-073`).
#[derive(Clone, Debug)]
pub struct StreamParticles3DData {
    pub(crate) version: u64,
    streamlines: Streamline3DData,
    particles_per_line: usize,
}

impl StreamParticles3DData {
    pub fn from_streamlines(streamlines: &Streamline3DData, particles_per_line: usize) -> Self {
        Self {
            version: crate::data::points::next_version(),
            streamlines: streamlines.clone(),
            particles_per_line: particles_per_line.clamp(1, 32),
        }
    }
    pub fn streamlines(&self) -> &Streamline3DData {
        &self.streamlines
    }
    pub fn particles_per_line(&self) -> usize {
        self.particles_per_line
    }
    pub fn sample_bytes(&self) -> usize {
        self.streamlines.sample_bytes()
    }
    pub(crate) fn allocation(&self) -> (usize, usize) {
        self.streamlines.allocation()
    }
    pub(crate) fn extent(&self, scales: [AxisScale; 3]) -> Option<[Point3D; 2]> {
        self.streamlines.extent(scales)
    }
    pub(crate) fn domain_invalid(&self, scales: [AxisScale; 3]) -> usize {
        self.streamlines.domain_invalid(scales)
    }
}

/// Multi-pass idempotent particle animation controller (`streamparticles`, `ML-073`).
#[derive(Clone, Copy, Debug)]
pub struct StreamParticlesPlayback {
    phase: f64,
    speed_cycles_per_sec: f64,
    playing: bool,
    last_frame: Option<u64>,
}

impl Default for StreamParticlesPlayback {
    fn default() -> Self {
        Self::new(0.45)
    }
}

impl StreamParticlesPlayback {
    pub fn new(speed_cycles_per_sec: f64) -> Self {
        Self {
            phase: 0.0,
            speed_cycles_per_sec: if speed_cycles_per_sec.is_finite() {
                speed_cycles_per_sec.max(0.0)
            } else {
                0.45
            },
            playing: true,
            last_frame: None,
        }
    }
    pub fn phase(&self) -> f64 {
        self.phase
    }
    pub fn set_phase(&mut self, phase: f64) {
        if phase.is_finite() {
            self.phase = phase.rem_euclid(1.0);
        }
    }
    pub fn set_playing(&mut self, playing: bool) {
        self.playing = playing;
    }
    pub fn is_playing(&self) -> bool {
        self.playing
    }

    pub fn advance_once(&mut self, frame_nr: u64, dt_secs: f64) -> bool {
        if !self.playing
            || self.last_frame == Some(frame_nr)
            || !dt_secs.is_finite()
            || dt_secs <= 0.0
        {
            return false;
        }
        self.last_frame = Some(frame_nr);
        self.phase = (self.phase + dt_secs * self.speed_cycles_per_sec).rem_euclid(1.0);
        true
    }

    pub fn sync_repaint(&self, ctx: &Context, interval: Duration) {
        if self.playing {
            ctx.request_repaint_after(interval);
        }
    }

    pub fn positions(&self, data: &StreamParticles3DData) -> Vec<(Point3D, f64)> {
        let m = data.particles_per_line;
        let mut out = Vec::with_capacity(data.streamlines.lines().len() * m);
        for line in data.streamlines.lines() {
            let n = line.points.len();
            if n < 2 {
                continue;
            }
            for k in 0..m {
                let u = (self.phase + k as f64 / m as f64).rem_euclid(1.0) * (n - 1) as f64;
                let idx = (u.floor() as usize).min(n - 2);
                let f = u - idx as f64;
                let (a, b) = (line.points[idx], line.points[idx + 1]);
                let p = [
                    a[0] + f * (b[0] - a[0]),
                    a[1] + f * (b[1] - a[1]),
                    a[2] + f * (b[2] - a[2]),
                ];
                let spd = line.speeds[idx] + f * (line.speeds[idx + 1] - line.speeds[idx]);
                out.push((p, spd));
            }
        }
        out
    }
}
