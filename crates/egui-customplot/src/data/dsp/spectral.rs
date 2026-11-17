use std::sync::Arc;

use super::fft::{WindowKind, fft_arbitrary, fft_radix2};
use crate::{AxisScale, GridData, PlotBounds, PlotData, PlotPoint};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DspError {
    Empty,
    InvalidSampleRate,
    InvalidWindow,
    NonFinite,
    InvalidFrequencyRange,
}

impl std::fmt::Display for DspError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Empty => f.write_str("signal must not be empty"),
            Self::InvalidSampleRate => f.write_str("sample rate fs_hz must be finite and positive"),
            Self::InvalidWindow => f.write_str("invalid window/segment/overlap configuration"),
            Self::NonFinite => f.write_str("signal samples must be finite"),
            Self::InvalidFrequencyRange => {
                f.write_str("frequency bounds must satisfy 0 < f_min < f_max <= fs/2")
            }
        }
    }
}

impl std::error::Error for DspError {}

/// Power spectral density dataset (`pwelch` / `periodogram` / `pspectrum`, `EXT-22`).
#[derive(Clone, Debug)]
pub struct PspectrumData {
    freqs_hz: Arc<[f64]>,
    psd_linear: Arc<[f64]>,
    psd_db_curve: PlotData,
}

impl PspectrumData {
    pub fn periodogram(signal: &[f64], fs_hz: f64, window: WindowKind) -> Result<Self, DspError> {
        Self::welch(signal, fs_hz, signal.len(), 0, window)
    }

    pub fn welch(
        signal: &[f64],
        fs_hz: f64,
        segment_len: usize,
        overlap: usize,
        window: WindowKind,
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
        if segment_len < 4 || segment_len > signal.len() || overlap >= segment_len {
            return Err(DspError::InvalidWindow);
        }

        let weights = window.weights(segment_len);
        let u: f64 = weights.iter().map(|w| w * w).sum::<f64>().max(1e-18);
        let step = (segment_len - overlap).max(1);
        let n_bins = segment_len / 2 + 1;
        let mut accum = vec![0.0_f64; n_bins];
        let mut n_segs = 0usize;

        let is_pow2 = segment_len.is_power_of_two();
        let mut start = 0usize;
        while start + segment_len <= signal.len() {
            let mut buf: Vec<[f64; 2]> = signal[start..start + segment_len]
                .iter()
                .zip(&weights)
                .map(|(&x, &w)| [x * w, 0.0])
                .collect();
            let spec = if is_pow2 {
                fft_radix2(&mut buf, false);
                buf
            } else {
                fft_arbitrary(&buf, false)
            };
            for k in 0..n_bins {
                let p = spec[k][0] * spec[k][0] + spec[k][1] * spec[k][1];
                let factor = if k == 0 || (segment_len.is_multiple_of(2) && k == n_bins - 1) {
                    1.0
                } else {
                    2.0
                };
                accum[k] += factor * p / (fs_hz * u);
            }
            n_segs += 1;
            start += step;
        }
        let inv_segs = 1.0 / (n_segs.max(1) as f64);
        let df = fs_hz / segment_len as f64;
        let mut freqs = Vec::with_capacity(n_bins);
        let mut psd_lin = Vec::with_capacity(n_bins);
        let mut pts: Vec<PlotPoint> = Vec::with_capacity(n_bins);
        for (k, v) in accum.into_iter().enumerate() {
            let f = k as f64 * df;
            let p = v * inv_segs;
            let db = 10.0 * p.max(1e-16).log10();
            freqs.push(f);
            psd_lin.push(p);
            pts.push([f, db]);
        }

        Ok(Self {
            freqs_hz: freqs.into(),
            psd_linear: psd_lin.into(),
            psd_db_curve: PlotData::new(pts),
        })
    }

    pub fn freqs_hz(&self) -> &[f64] {
        &self.freqs_hz
    }
    pub fn psd_linear(&self) -> &[f64] {
        &self.psd_linear
    }
    pub fn psd_db_curve(&self) -> &PlotData {
        &self.psd_db_curve
    }
    pub fn sample_bytes(&self) -> usize {
        self.psd_db_curve.sample_bytes() + self.psd_linear.len() * 8
    }
}

/// Time-frequency STFT power spectrogram (`spectrogram`, `EXT-22`).
#[derive(Clone, Debug)]
pub struct SpectrogramData {
    grid: GridData,
    value_range: [f64; 2],
}

impl SpectrogramData {
    pub fn stft(
        signal: &[f64],
        fs_hz: f64,
        window_len: usize,
        overlap: usize,
        nfft: usize,
        window: WindowKind,
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
        if window_len < 4 || window_len > signal.len() || overlap >= window_len || nfft < window_len
        {
            return Err(DspError::InvalidWindow);
        }

        let weights = window.weights(window_len);
        let u: f64 = weights.iter().map(|w| w * w).sum::<f64>().max(1e-18);
        let step = (window_len - overlap).max(1);
        let n_freqs = nfft / 2 + 1;
        let is_pow2 = nfft.is_power_of_two();

        let mut centers = Vec::new();
        let mut cols: Vec<Vec<f64>> = Vec::new();
        let mut start = 0usize;
        while start + window_len <= signal.len() {
            let mut buf = vec![[0.0_f64; 2]; nfft];
            for (i, (&x, &w)) in signal[start..start + window_len]
                .iter()
                .zip(&weights)
                .enumerate()
            {
                buf[i][0] = x * w;
            }
            let spec = if is_pow2 {
                fft_radix2(&mut buf, false);
                buf
            } else {
                fft_arbitrary(&buf, false)
            };
            let mut col = Vec::with_capacity(n_freqs);
            for (k, val) in spec.iter().take(n_freqs).enumerate() {
                let factor = if k == 0 || (nfft.is_multiple_of(2) && k == n_freqs - 1) {
                    1.0
                } else {
                    2.0
                };
                let p = factor * (val[0] * val[0] + val[1] * val[1]) / (fs_hz * u);
                col.push((10.0 * p.max(1e-14).log10()).max(-140.0));
            }
            centers.push((start as f64 + 0.5 * window_len as f64) / fs_hz);
            cols.push(col);
            start += step;
        }
        if centers.is_empty() {
            return Err(DspError::InvalidWindow);
        }

        let n_cols = cols.len();
        let mut x_edges = Vec::with_capacity(n_cols + 1);
        let dt = step as f64 / fs_hz;
        x_edges.push((centers[0] - 0.5 * dt).max(0.0));
        for w in centers.windows(2) {
            x_edges.push(0.5 * (w[0] + w[1]));
        }
        x_edges.push(centers[n_cols - 1] + 0.5 * dt);

        let df = fs_hz / nfft as f64;
        let y_edges: Vec<f64> = (0..=n_freqs)
            .map(|k| (k as f64 - 0.5).max(0.0) * df)
            .collect();
        let mut values = Vec::with_capacity(n_cols * n_freqs);
        let mut vmin = f64::INFINITY;
        let mut vmax = f64::NEG_INFINITY;
        for row in 0..n_freqs {
            for col in &cols {
                let db = col[row];
                vmin = vmin.min(db);
                vmax = vmax.max(db);
                values.push(db);
            }
        }
        let value_range = if (vmax - vmin).abs() > 1e-6 {
            [vmin, vmax]
        } else {
            [vmin - 1.0, vmax + 1.0]
        };
        let grid = GridData::new(x_edges, y_edges, values).map_err(|_| DspError::InvalidWindow)?;
        Ok(Self { grid, value_range })
    }

    pub fn grid(&self) -> &GridData {
        &self.grid
    }
    pub fn value_range(&self) -> [f64; 2] {
        self.value_range
    }
    pub fn bounds(&self, scales: [AxisScale; 2]) -> Option<PlotBounds> {
        let (min, max) = self.grid.extent(scales)?;
        PlotBounds::new(min, max).ok()
    }
}
