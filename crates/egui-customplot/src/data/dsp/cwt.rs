use super::{fft::fft_radix2, spectral::DspError};
use crate::{AxisScale, GridData, PlotBounds, PlotData, PlotPoint};

/// Continuous Wavelet Transform scalogram using analytic Morlet wavelet (`omega0 = 6.0`, `EXT-22`).
#[derive(Clone, Debug)]
pub struct CwtData {
    grid: GridData,
    coi_left: PlotData,
    coi_right: PlotData,
    value_range: [f64; 2],
}

impl CwtData {
    pub fn morlet_logspace(
        signal: &[f64],
        fs_hz: f64,
        f_min: f64,
        f_max: f64,
        n_freqs: usize,
    ) -> Result<Self, DspError> {
        if signal.is_empty() {
            return Err(DspError::Empty);
        }
        if !fs_hz.is_finite() || fs_hz <= 0.0 {
            return Err(DspError::InvalidSampleRate);
        }
        if signal.iter().any(|v| !v.is_finite()) {
            return Err(DspError::NonFinite);
        }
        if !f_min.is_finite()
            || !f_max.is_finite()
            || f_min <= 0.0
            || f_max <= f_min
            || f_max > 0.5 * fs_hz
        {
            return Err(DspError::InvalidFrequencyRange);
        }

        let n = signal.len();
        let n_fft = (n * 2).next_power_of_two();
        let n_sc = n_freqs.clamp(6, 96);
        let mean = signal.iter().sum::<f64>() / n as f64;
        let mut x_hat = vec![[0.0_f64; 2]; n_fft];
        for (i, &x) in signal.iter().enumerate() {
            x_hat[i][0] = x - mean;
        }
        fft_radix2(&mut x_hat, false);

        const OMEGA0: f64 = 6.0;
        let morlet_norm = std::f64::consts::PI.powf(-0.25);
        // Fourier factor connecting Morlet scale s to frequency f: f = (omega0 + sqrt(2 + omega0^2)) / (4 * pi * s)
        let fourier_factor =
            (4.0 * std::f64::consts::PI) / (OMEGA0 + (2.0 + OMEGA0 * OMEGA0).sqrt());
        let l0 = f_min.ln();
        let l1 = f_max.ln();
        let freqs: Vec<f64> = (0..n_sc)
            .map(|k| (l0 + (l1 - l0) * (k as f64 / (n_sc - 1) as f64)).exp())
            .collect();

        let dt = 1.0 / fs_hz;
        let mut ang_freqs = vec![0.0_f64; n_fft];
        for (k, w) in ang_freqs.iter_mut().enumerate() {
            let signed_k = if k <= n_fft / 2 {
                k as f64
            } else {
                k as f64 - n_fft as f64
            };
            *w = std::f64::consts::TAU * signed_k / (n_fft as f64 * dt);
        }

        let mut values = Vec::with_capacity(n_sc * n);
        let mut vmin = f64::INFINITY;
        let mut vmax = f64::NEG_INFINITY;
        let mut work = vec![[0.0_f64; 2]; n_fft];

        for &freq in &freqs {
            let scale = 1.0 / (fourier_factor * freq);
            let scale_norm = (std::f64::consts::TAU * scale / dt).sqrt() * morlet_norm;
            for (k, wk) in ang_freqs.iter().enumerate() {
                if *wk > 0.0 {
                    let arg = scale * *wk - OMEGA0;
                    let psi = scale_norm * (-0.5 * arg * arg).exp();
                    work[k] = [x_hat[k][0] * psi, x_hat[k][1] * psi];
                } else {
                    work[k] = [0.0, 0.0];
                }
            }
            fft_radix2(&mut work, true);
            for w in work.iter().take(n) {
                let mag = w[0].hypot(w[1]);
                vmin = vmin.min(mag);
                vmax = vmax.max(mag);
                values.push(mag);
            }
        }

        let x_edges: Vec<f64> = (0..=n).map(|i| i as f64 * dt).collect();
        let mut y_edges = Vec::with_capacity(n_sc + 1);
        y_edges.push((freqs[0] * 0.92).max(1e-6));
        for w in freqs.windows(2) {
            y_edges.push((w[0] * w[1]).sqrt());
        }
        y_edges.push(freqs[n_sc - 1] * 1.08);

        let total_t = (n - 1) as f64 * dt;
        let mut left_pts: Vec<PlotPoint> = Vec::with_capacity(n_sc);
        let mut right_pts: Vec<PlotPoint> = Vec::with_capacity(n_sc);
        for &freq in &freqs {
            // Morlet e-folding COI half-width: dt_coi = sqrt(2) * scale
            let t_coi = (std::f64::consts::SQRT_2 / (fourier_factor * freq)).min(0.5 * total_t);
            left_pts.push([t_coi, freq]);
            right_pts.push([(total_t - t_coi).max(0.0), freq]);
        }

        let grid = GridData::new(x_edges, y_edges, values).map_err(|_| DspError::InvalidWindow)?;
        let value_range = if (vmax - vmin).abs() > 1e-12 {
            [vmin, vmax]
        } else {
            [0.0, vmax.max(1.0)]
        };
        Ok(Self {
            grid,
            coi_left: PlotData::new_unindexed(left_pts),
            coi_right: PlotData::new_unindexed(right_pts),
            value_range,
        })
    }

    pub fn grid(&self) -> &GridData {
        &self.grid
    }
    pub fn coi_left(&self) -> &PlotData {
        &self.coi_left
    }
    pub fn coi_right(&self) -> &PlotData {
        &self.coi_right
    }
    pub fn value_range(&self) -> [f64; 2] {
        self.value_range
    }
    pub fn sample_bytes(&self) -> usize {
        self.grid.sample_bytes() + self.coi_left.sample_bytes() + self.coi_right.sample_bytes()
    }
    pub fn bounds(&self, scales: [AxisScale; 2]) -> Option<PlotBounds> {
        let (min, max) = self.grid.extent(scales)?;
        PlotBounds::new(min, max).ok()
    }
}
