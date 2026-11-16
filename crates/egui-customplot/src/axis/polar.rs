use std::f64::consts::{FRAC_PI_2, PI, TAU};

use egui::Pos2;

use super::{AxisTick, BoundsError};

/// Location of 0 radians / 0° on the perimeter of a polar plot.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ThetaZeroLocation {
    #[default]
    Right,
    Top,
    Left,
    Bottom,
}

/// Positive angular sweep direction around the polar origin.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ThetaDirection {
    #[default]
    CounterClockwise,
    Clockwise,
}

/// How negative `r` values are mapped onto a polar plot.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum NegativeRadiusMode {
    /// Modern R2026b `polarplot` behavior when auto-ranging includes negative `r`: shift `r_min <= 0`.
    #[default]
    ShiftMinRadius,
    /// Legacy MATLAB `polar` (`LG-004`) behavior: reflect across the origin as `(theta + pi, |r|)`.
    ReflectOrigin,
    /// Keep `(theta, r)` unchanged and clip any radius `< r_lim[0]`.
    ClipBelowMin,
}

/// Polar-to-screen coordinate transform matching MATLAB polar axes semantics.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PolarTransform {
    center: Pos2,
    pixel_radius: f32,
    r_lim: [f64; 2],
    theta_zero: ThetaZeroLocation,
    theta_dir: ThetaDirection,
    theta_lim: [f64; 2],
    theta_offset: f64,
}

impl PolarTransform {
    pub fn new(
        center: Pos2,
        pixel_radius: f32,
        r_lim: [f64; 2],
        theta_zero: ThetaZeroLocation,
        theta_dir: ThetaDirection,
        theta_lim: [f64; 2],
        theta_offset: f64,
    ) -> Result<Self, BoundsError> {
        if !center.is_finite()
            || !pixel_radius.is_finite()
            || pixel_radius < 1.0
            || !r_lim[0].is_finite()
            || !r_lim[1].is_finite()
            || r_lim[1] <= r_lim[0]
            || !theta_lim[0].is_finite()
            || !theta_lim[1].is_finite()
            || !theta_offset.is_finite()
        {
            return Err(BoundsError);
        }
        let span = theta_lim[1] - theta_lim[0];
        if span <= 1e-9 || span > TAU + 1e-6 {
            return Err(BoundsError);
        }
        Ok(Self {
            center,
            pixel_radius,
            r_lim,
            theta_zero,
            theta_dir,
            theta_lim: [theta_lim[0], theta_lim[0] + span.min(TAU)],
            theta_offset,
        })
    }

    pub fn center(&self) -> Pos2 {
        self.center
    }

    pub fn pixel_radius(&self) -> f32 {
        self.pixel_radius
    }

    pub fn r_lim(&self) -> [f64; 2] {
        self.r_lim
    }

    pub fn theta_zero(&self) -> ThetaZeroLocation {
        self.theta_zero
    }

    pub fn theta_direction(&self) -> ThetaDirection {
        self.theta_dir
    }

    pub fn theta_lim(&self) -> [f64; 2] {
        self.theta_lim
    }

    pub fn is_full_circle(&self) -> bool {
        (self.theta_lim[1] - self.theta_lim[0]) >= TAU - 1e-6
    }

    /// Maps mathematical angle `theta_rad` to egui screen radians (`+x` right, `+y` down).
    pub fn screen_angle(&self, theta_rad: f64) -> f64 {
        let zero = match self.theta_zero {
            ThetaZeroLocation::Right => 0.0,
            ThetaZeroLocation::Top => -FRAC_PI_2,
            ThetaZeroLocation::Left => PI,
            ThetaZeroLocation::Bottom => FRAC_PI_2,
        };
        let sign = match self.theta_dir {
            ThetaDirection::CounterClockwise => -1.0,
            ThetaDirection::Clockwise => 1.0,
        };
        zero + sign * (theta_rad + self.theta_offset)
    }

    /// Applies `neg_mode` to `(theta, r)`, returning adjusted `(theta, r)`.
    pub fn apply_negative_mode(theta: f64, r: f64, neg_mode: NegativeRadiusMode) -> (f64, f64) {
        if r < 0.0 && matches!(neg_mode, NegativeRadiusMode::ReflectOrigin) {
            (theta + PI, -r)
        } else {
            (theta, r)
        }
    }

    /// Normalizes radius `r` to `[0.0, 1.0]` relative to `r_lim` without clamping.
    pub fn normalized_radius(&self, r: f64) -> f64 {
        (r - self.r_lim[0]) / (self.r_lim[1] - self.r_lim[0])
    }

    /// True if `theta` lies inside `self.theta_lim` (modulo `2*pi`).
    pub fn contains_theta(&self, theta: f64) -> bool {
        if !theta.is_finite() {
            return false;
        }
        if self.is_full_circle() {
            return true;
        }
        let span = self.theta_lim[1] - self.theta_lim[0];
        let delta = (theta - self.theta_lim[0]).rem_euclid(TAU);
        delta <= span + 1e-9 || (TAU - delta) <= 1e-9
    }

    /// Screen position even if `r` extends outside `r_lim` (useful for radial clipping).
    pub fn unclipped_position_from_polar(
        &self,
        theta: f64,
        r: f64,
        neg_mode: NegativeRadiusMode,
    ) -> Option<Pos2> {
        if !theta.is_finite() || !r.is_finite() {
            return None;
        }
        let (theta, r) = Self::apply_negative_mode(theta, r, neg_mode);
        let rho_px = (self.normalized_radius(r) * f64::from(self.pixel_radius)) as f32;
        if !rho_px.is_finite() {
            return None;
        }
        let ang = self.screen_angle(theta);
        Some(Pos2::new(
            self.center.x + rho_px * ang.cos() as f32,
            self.center.y + rho_px * ang.sin() as f32,
        ))
    }

    /// Screen position only when within both `[r_lim[0], r_lim[1]]` and `theta_lim`.
    pub fn position_from_polar(
        &self,
        theta: f64,
        r: f64,
        neg_mode: NegativeRadiusMode,
    ) -> Option<Pos2> {
        if !theta.is_finite() || !r.is_finite() {
            return None;
        }
        let (theta, r) = Self::apply_negative_mode(theta, r, neg_mode);
        if r < self.r_lim[0] - 1e-9 || r > self.r_lim[1] + 1e-9 || !self.contains_theta(theta) {
            return None;
        }
        let rho_px =
            (self.normalized_radius(r).clamp(0.0, 1.0) * f64::from(self.pixel_radius)) as f32;
        let ang = self.screen_angle(theta);
        Some(Pos2::new(
            self.center.x + rho_px * ang.cos() as f32,
            self.center.y + rho_px * ang.sin() as f32,
        ))
    }

    /// Inverse mapping from screen position back to `[theta_rad, r]`.
    pub fn polar_from_position(&self, pos: Pos2) -> Option<[f64; 2]> {
        if !pos.is_finite() {
            return None;
        }
        let dx = f64::from(pos.x - self.center.x);
        let dy = f64::from(pos.y - self.center.y);
        let dist = dx.hypot(dy);
        if dist > f64::from(self.pixel_radius) + 1e-2 {
            return None;
        }
        let t = dist / f64::from(self.pixel_radius);
        let r = self.r_lim[0] + t * (self.r_lim[1] - self.r_lim[0]);
        let zero = match self.theta_zero {
            ThetaZeroLocation::Right => 0.0,
            ThetaZeroLocation::Top => -FRAC_PI_2,
            ThetaZeroLocation::Left => PI,
            ThetaZeroLocation::Bottom => FRAC_PI_2,
        };
        let sign = match self.theta_dir {
            ThetaDirection::CounterClockwise => -1.0,
            ThetaDirection::Clockwise => 1.0,
        };
        let screen_ang = dy.atan2(dx);
        let raw_theta = sign * (screen_ang - zero) - self.theta_offset;
        let span = self.theta_lim[1] - self.theta_lim[0];
        let theta = if self.is_full_circle() {
            raw_theta.rem_euclid(TAU)
        } else {
            let delta = (raw_theta - self.theta_lim[0]).rem_euclid(TAU);
            if delta > span + 1e-6 && (TAU - delta) > 1e-6 {
                return None;
            }
            self.theta_lim[0] + delta.min(span)
        };
        Some([theta, r])
    }

    pub fn radial_ticks(&self, max_ticks: usize) -> Vec<AxisTick> {
        let count = max_ticks.clamp(2, 8);
        let span = self.r_lim[1] - self.r_lim[0];
        let raw_step = span / count as f64;
        let exp = 10.0_f64.powf(raw_step.log10().floor());
        let f = raw_step / exp;
        let nice = if f <= 1.5 {
            1.0
        } else if f <= 3.0 {
            2.0
        } else if f <= 7.0 {
            5.0
        } else {
            10.0
        };
        let step = nice * exp;
        let start_k = (self.r_lim[0] / step).ceil() as i64;
        let end_k = (self.r_lim[1] / step).floor() as i64;
        let mut ticks = Vec::new();
        for k in start_k..=end_k {
            let v = (k as f64 * step).clamp(self.r_lim[0], self.r_lim[1]);
            let label = if (v - v.round()).abs() < 1e-6 {
                format!("{:.0}", v.round())
            } else {
                format!("{v:.3}")
                    .trim_end_matches('0')
                    .trim_end_matches('.')
                    .to_owned()
            };
            ticks.push(AxisTick {
                value: v,
                label,
                major: true,
            });
        }
        ticks
    }

    pub fn angular_ticks(&self) -> Vec<(f64, String)> {
        if self.is_full_circle() {
            return (0..12)
                .map(|i| {
                    let deg = i * 30;
                    (f64::from(deg).to_radians(), format!("{deg}°"))
                })
                .collect();
        }
        let deg0 = self.theta_lim[0].to_degrees();
        let deg1 = self.theta_lim[1].to_degrees();
        let step = 30.0_f64;
        let start = (deg0 / step).ceil() as i32 * 30;
        let end = (deg1 / step).floor() as i32 * 30;
        let mut out = Vec::new();
        for deg in (start..=end).step_by(30) {
            out.push((f64::from(deg).to_radians(), format!("{deg}°")));
        }
        if out.is_empty() {
            out.push((self.theta_lim[0], format!("{deg0:.0}°")));
            out.push((self.theta_lim[1], format!("{deg1:.0}°")));
        }
        out
    }

    pub fn zoom_radial(&self, factor: f64, lock_r_zero: bool) -> [f64; 2] {
        let factor = factor.clamp(0.2, 5.0);
        let [lo, hi] = self.r_lim;
        if lock_r_zero && lo >= 0.0 {
            let span = (hi - lo).max(1e-9);
            [lo, lo + span / factor]
        } else {
            let mid = 0.5 * (lo + hi);
            let half = 0.5 * (hi - lo) / factor;
            [mid - half, mid + half]
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn orientations_and_inverse_roundtrip_match() {
        let cases = [
            (
                ThetaZeroLocation::Right,
                ThetaDirection::CounterClockwise,
                0.0,
                Pos2::new(200.0, 100.0),
                Pos2::new(100.0, 0.0),
            ),
            (
                ThetaZeroLocation::Top,
                ThetaDirection::Clockwise,
                0.0,
                Pos2::new(100.0, 0.0),
                Pos2::new(200.0, 100.0),
            ),
            (
                ThetaZeroLocation::Left,
                ThetaDirection::Clockwise,
                0.0,
                Pos2::new(0.0, 100.0),
                Pos2::new(100.0, 0.0),
            ),
            (
                ThetaZeroLocation::Bottom,
                ThetaDirection::CounterClockwise,
                0.0,
                Pos2::new(100.0, 200.0),
                Pos2::new(200.0, 100.0),
            ),
        ];
        for (zero, dir, offset, exp_zero, exp_half_pi) in cases {
            let t = PolarTransform::new(
                Pos2::new(100.0, 100.0),
                100.0,
                [0.0, 2.0],
                zero,
                dir,
                [0.0, TAU],
                offset,
            )
            .unwrap();
            let p0 = t
                .position_from_polar(0.0, 2.0, NegativeRadiusMode::ShiftMinRadius)
                .unwrap();
            assert!(p0.distance(exp_zero) < 1e-3, "{zero:?} {dir:?}: {p0:?}");
            let p90 = t
                .position_from_polar(FRAC_PI_2, 2.0, NegativeRadiusMode::ShiftMinRadius)
                .unwrap();
            assert!(
                p90.distance(exp_half_pi) < 1e-3,
                "{zero:?} {dir:?}: {p90:?}"
            );
            let [rt, rr] = t.polar_from_position(p90).unwrap();
            assert!((rt - FRAC_PI_2).abs() < 1e-4);
            assert!((rr - 2.0).abs() < 1e-4);
        }
    }
}
