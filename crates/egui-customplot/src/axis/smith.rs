use egui::{Pos2, Rect, Vec2, pos2};

/// Smith chart grid mode (`EXT-19`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SmithGridKind {
    #[default]
    Impedance,
    Admittance,
    ImpedanceAdmittance,
}

/// Conformal Möbius / bilinear transform between normalized impedance `z = r + jx`
/// and complex reflection coefficient `Gamma = (z - 1) / (z + 1)` inside the viewport disk.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SmithTransform {
    disk_rect: Rect,
    center: [f64; 2],
    half_span: f64,
}

impl SmithTransform {
    pub fn new(plot_area: Rect, center: [f64; 2], half_span: f64) -> Self {
        let side = plot_area.width().min(plot_area.height()).max(4.0);
        let disk_rect = Rect::from_center_size(plot_area.center(), Vec2::splat(side));
        let cx = if center[0].is_finite() {
            center[0]
        } else {
            0.0
        };
        let cy = if center[1].is_finite() {
            center[1]
        } else {
            0.0
        };
        let span = if half_span.is_finite() && half_span > 0.0 {
            half_span.clamp(1e-4, 20.0)
        } else {
            1.08
        };
        Self {
            disk_rect,
            center: [cx, cy],
            half_span: span,
        }
    }

    pub fn disk_rect(self) -> Rect {
        self.disk_rect
    }
    pub fn center_gamma(self) -> [f64; 2] {
        self.center
    }
    pub fn half_span(self) -> f64 {
        self.half_span
    }

    /// Conformal map `z = r + jx -> Gamma = (z - 1) / (z + 1)`.
    pub fn z_to_gamma(z: [f64; 2]) -> Option<[f64; 2]> {
        if !z[0].is_finite() || !z[1].is_finite() {
            return None;
        }
        let dr = z[0] + 1.0;
        let di = z[1];
        let denom = dr * dr + di * di;
        if denom < 1e-24 || !denom.is_finite() {
            return None;
        }
        let nr = z[0] - 1.0;
        let ni = z[1];
        Some([(nr * dr + ni * di) / denom, (ni * dr - nr * di) / denom])
    }

    /// Conformal inverse `Gamma -> z = (1 + Gamma) / (1 - Gamma)`.
    pub fn gamma_to_z(gamma: [f64; 2]) -> Option<[f64; 2]> {
        if !gamma[0].is_finite() || !gamma[1].is_finite() {
            return None;
        }
        let dr = 1.0 - gamma[0];
        let di = -gamma[1];
        let denom = dr * dr + di * di;
        if denom < 1e-24 || !denom.is_finite() {
            return None;
        }
        let nr = 1.0 + gamma[0];
        let ni = gamma[1];
        Some([(nr * dr + ni * di) / denom, (ni * dr - nr * di) / denom])
    }

    pub fn vswr_from_gamma(gamma: [f64; 2]) -> f64 {
        let mag = gamma[0].hypot(gamma[1]);
        if !mag.is_finite() {
            return f64::INFINITY;
        }
        if mag >= 1.0 - 1e-12 {
            f64::INFINITY
        } else {
            (1.0 + mag) / (1.0 - mag)
        }
    }

    pub fn return_loss_db(gamma: [f64; 2]) -> f64 {
        let mag = gamma[0].hypot(gamma[1]);
        if !mag.is_finite() || mag <= 1e-15 {
            f64::INFINITY
        } else {
            -20.0 * mag.log10()
        }
    }

    pub fn gamma_to_pos(self, gamma: [f64; 2]) -> Pos2 {
        let r_px = 0.5 * f64::from(self.disk_rect.width());
        let scale = r_px / self.half_span;
        let c = self.disk_rect.center();
        pos2(
            c.x + ((gamma[0] - self.center[0]) * scale) as f32,
            c.y - ((gamma[1] - self.center[1]) * scale) as f32,
        )
    }

    pub fn pos_to_gamma(self, pos: Pos2) -> [f64; 2] {
        let r_px = (0.5 * f64::from(self.disk_rect.width())).max(1.0);
        let scale = self.half_span / r_px;
        let c = self.disk_rect.center();
        [
            self.center[0] + f64::from(pos.x - c.x) * scale,
            self.center[1] - f64::from(pos.y - c.y) * scale,
        ]
    }

    pub fn pan(self, delta_px: Vec2) -> ([f64; 2], f64) {
        let r_px = (0.5 * f64::from(self.disk_rect.width())).max(1.0);
        let scale = self.half_span / r_px;
        let next = [
            (self.center[0] - f64::from(delta_px.x) * scale).clamp(-2.5, 2.5),
            (self.center[1] + f64::from(delta_px.y) * scale).clamp(-2.5, 2.5),
        ];
        (next, self.half_span)
    }

    pub fn zoom_at(self, anchor_pos: Pos2, factor: f64) -> ([f64; 2], f64) {
        let f = if factor.is_finite() && factor > 0.0 {
            factor
        } else {
            1.0
        };
        let anchor = self.pos_to_gamma(anchor_pos);
        let next_span = (self.half_span * f).clamp(0.05, 4.0);
        let ratio = next_span / self.half_span;
        let next_center = [
            (anchor[0] + (self.center[0] - anchor[0]) * ratio).clamp(-2.5, 2.5),
            (anchor[1] + (self.center[1] - anchor[1]) * ratio).clamp(-2.5, 2.5),
        ];
        (next_center, next_span)
    }

    /// Generates points along a constant normalized resistance circle `z = r + jx`, `x in [-40, 40]`.
    pub fn constant_resistance_circle(r: f64, steps: usize) -> Vec<[f64; 2]> {
        let center_x = r / (r + 1.0);
        let rad = 1.0 / (r + 1.0);
        let n = steps.max(16);
        (0..=n)
            .map(|k| {
                let th = std::f64::consts::TAU * (k as f64 / n as f64);
                [center_x + rad * th.cos(), rad * th.sin()]
            })
            .collect()
    }

    /// Generates points along a constant normalized reactance arc inside `|Gamma| <= 1`.
    pub fn constant_reactance_arc(x: f64, steps: usize) -> Vec<[f64; 2]> {
        if x.abs() < 1e-12 {
            return vec![[-1.0, 0.0], [1.0, 0.0]];
        }
        // Sample parameterized by r from 0 to 250 using non-uniform density near r=1.
        let n = steps.max(16);
        (0..=n)
            .filter_map(|k| {
                let u = k as f64 / n as f64;
                let r = (u / (1.0 - u + 1e-3)) * 0.5;
                Self::z_to_gamma([r, x])
            })
            .chain(std::iter::once([1.0, 0.0]))
            .collect()
    }
}
