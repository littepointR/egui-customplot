use std::sync::Arc;

use super::poly::{ControlError, TransferFunction, c_abs};
use crate::{PlotData, PlotPoint};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct StabilityMargins {
    pub gain_margin_db: Option<f64>,
    pub phase_crossover_rad_s: Option<f64>,
    pub phase_margin_deg: Option<f64>,
    pub gain_crossover_rad_s: Option<f64>,
}

/// Complex frequency response `H(j*w)` powering Bode, Nyquist, and Nichols plots (`EXT-21`).
#[derive(Clone, Debug)]
pub struct FrequencyResponseData {
    omega: Arc<[f64]>,
    response: Arc<[[f64; 2]]>,
    mag_curve: PlotData,
    phase_curve: PlotData,
    nyquist_curve: PlotData,
    nichols_curve: PlotData,
    margins: StabilityMargins,
}

impl FrequencyResponseData {
    pub fn from_transfer_fn(
        tf: &TransferFunction,
        w_min: f64,
        w_max: f64,
        n_points: usize,
    ) -> Result<Self, ControlError> {
        if !w_min.is_finite() || !w_max.is_finite() || w_min <= 0.0 || w_max <= w_min {
            return Err(ControlError::InvalidFrequencyRange);
        }
        let n = n_points.clamp(16, 8192);
        let l0 = w_min.ln();
        let l1 = w_max.ln();
        let mut omega = Vec::with_capacity(n);
        let mut resp = Vec::with_capacity(n);
        for k in 0..n {
            let t = k as f64 / (n - 1) as f64;
            let w = (l0 + (l1 - l0) * t).exp();
            let h = tf.eval_jw(w).ok_or(ControlError::InvalidPolynomial)?;
            omega.push(w);
            resp.push(h);
        }
        Self::from_samples(&omega, &resp)
    }

    pub fn from_samples(
        omega_rad_s: &[f64],
        resp_re_im: &[[f64; 2]],
    ) -> Result<Self, ControlError> {
        if omega_rad_s.is_empty() {
            return Err(ControlError::Empty);
        }
        if omega_rad_s.len() != resp_re_im.len() {
            return Err(ControlError::LengthMismatch);
        }
        if omega_rad_s.iter().any(|w| !w.is_finite() || *w <= 0.0)
            || resp_re_im
                .iter()
                .any(|h| !h[0].is_finite() || !h[1].is_finite())
        {
            return Err(ControlError::NonFinite);
        }
        if omega_rad_s.windows(2).any(|w| w[1] <= w[0]) {
            return Err(ControlError::InvalidFrequencyRange);
        }

        let n = omega_rad_s.len();
        let mut mag_pts: Vec<PlotPoint> = Vec::with_capacity(n);
        let mut phase_pts: Vec<PlotPoint> = Vec::with_capacity(n);
        let mut nyq_pts: Vec<PlotPoint> = Vec::with_capacity(n);
        let mut nichols_pts: Vec<PlotPoint> = Vec::with_capacity(n);

        let mut unwrapped_rad = resp_re_im[0][1].atan2(resp_re_im[0][0]);
        for i in 0..n {
            let h = resp_re_im[i];
            let mag = c_abs(h).max(1e-15);
            let mag_db = 20.0 * mag.log10();
            let raw = h[1].atan2(h[0]);
            if i > 0 {
                let prev_wrapped = unwrapped_rad.rem_euclid(std::f64::consts::TAU);
                let mut diff = raw - prev_wrapped.rem_euclid(std::f64::consts::TAU);
                while diff > std::f64::consts::PI {
                    diff -= std::f64::consts::TAU;
                }
                while diff < -std::f64::consts::PI {
                    diff += std::f64::consts::TAU;
                }
                unwrapped_rad += diff;
            }
            let phase_deg = unwrapped_rad.to_degrees();
            let w = omega_rad_s[i];
            mag_pts.push([w, mag_db]);
            phase_pts.push([w, phase_deg]);
            nyq_pts.push(h);
            nichols_pts.push([phase_deg, mag_db]);
        }

        let margins = compute_margins(&mag_pts, &phase_pts);
        Ok(Self {
            omega: Arc::from(omega_rad_s),
            response: Arc::from(resp_re_im),
            mag_curve: PlotData::new(mag_pts),
            phase_curve: PlotData::new(phase_pts),
            nyquist_curve: PlotData::new_unindexed(nyq_pts),
            nichols_curve: PlotData::new_unindexed(nichols_pts),
            margins,
        })
    }

    pub fn omega(&self) -> &[f64] {
        &self.omega
    }
    pub fn complex_response(&self) -> &[[f64; 2]] {
        &self.response
    }
    pub fn mag_curve(&self) -> &PlotData {
        &self.mag_curve
    }
    pub fn phase_curve(&self) -> &PlotData {
        &self.phase_curve
    }
    pub fn nyquist_curve(&self) -> &PlotData {
        &self.nyquist_curve
    }
    pub fn nichols_curve(&self) -> &PlotData {
        &self.nichols_curve
    }
    pub fn margins(&self) -> StabilityMargins {
        self.margins
    }
    pub fn sample_bytes(&self) -> usize {
        self.mag_curve.sample_bytes()
            + self.phase_curve.sample_bytes()
            + self.nyquist_curve.sample_bytes()
    }
}

fn compute_margins(mag_pts: &[PlotPoint], phase_pts: &[PlotPoint]) -> StabilityMargins {
    let mut margins = StabilityMargins::default();
    let mut min_gm = f64::INFINITY;
    let mut min_pm = f64::INFINITY;

    for i in 0..mag_pts.len().saturating_sub(1) {
        let [w0, m0] = mag_pts[i];
        let [w1, m1] = mag_pts[i + 1];
        let p0 = phase_pts[i][1];
        let p1 = phase_pts[i + 1][1];

        // Gain crossover (0 dB crossing -> Phase Margin).
        if (m0 >= 0.0 && m1 < 0.0) || (m0 <= 0.0 && m1 > 0.0) {
            let t = (-m0 / (m1 - m0)).clamp(0.0, 1.0);
            let w_gc = (w0.ln() + t * (w1.ln() - w0.ln())).exp();
            let p_gc = p0 + t * (p1 - p0);
            let mut pm = (p_gc + 180.0).rem_euclid(360.0);
            if pm > 180.0 {
                pm -= 360.0;
            }
            if pm.abs() < min_pm.abs() {
                min_pm = pm;
                margins.phase_margin_deg = Some(pm);
                margins.gain_crossover_rad_s = Some(w_gc);
            }
        }

        // Phase crossover (-180 deg mod 360 crossing -> Gain Margin).
        let a0 = (p0 + 180.0).rem_euclid(360.0) - 180.0;
        let a1 = (p1 + 180.0).rem_euclid(360.0) - 180.0;
        let _ = (a0, a1);
        let k_low = ((p0.min(p1) + 180.0) / 360.0).floor() as i64 - 1;
        let k_high = ((p0.max(p1) + 180.0) / 360.0).ceil() as i64 + 1;
        for k in k_low..=k_high {
            let target = -180.0 + (k as f64) * 360.0;
            if (p0 - target) * (p1 - target) <= 0.0 && (p1 - p0).abs() > 1e-12 {
                let t = ((target - p0) / (p1 - p0)).clamp(0.0, 1.0);
                let w_pc = (w0.ln() + t * (w1.ln() - w0.ln())).exp();
                let m_pc = m0 + t * (m1 - m0);
                let gm_db = -m_pc;
                if gm_db.abs() < min_gm.abs() || margins.gain_margin_db.is_none() {
                    min_gm = gm_db;
                    margins.gain_margin_db = Some(gm_db);
                    margins.phase_crossover_rad_s = Some(w_pc);
                }
            }
        }
    }
    margins
}
