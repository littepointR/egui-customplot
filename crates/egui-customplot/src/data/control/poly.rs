use std::sync::Arc;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ControlError {
    Empty,
    InvalidPolynomial,
    InvalidFrequencyRange,
    NonFinite,
    LengthMismatch,
}

impl std::fmt::Display for ControlError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Empty => f.write_str("control dataset must not be empty"),
            Self::InvalidPolynomial => {
                f.write_str("polynomial must have at least one finite non-zero leading coefficient")
            }
            Self::InvalidFrequencyRange => {
                f.write_str("frequency bounds must be positive finite with w_min < w_max")
            }
            Self::NonFinite => f.write_str("control inputs must be finite"),
            Self::LengthMismatch => f.write_str("frequency and response slice lengths differ"),
        }
    }
}

impl std::error::Error for ControlError {}

#[inline]
pub(crate) fn c_add(a: [f64; 2], b: [f64; 2]) -> [f64; 2] {
    [a[0] + b[0], a[1] + b[1]]
}
#[inline]
pub(crate) fn c_sub(a: [f64; 2], b: [f64; 2]) -> [f64; 2] {
    [a[0] - b[0], a[1] - b[1]]
}
#[inline]
pub(crate) fn c_mul(a: [f64; 2], b: [f64; 2]) -> [f64; 2] {
    [a[0] * b[0] - a[1] * b[1], a[0] * b[1] + a[1] * b[0]]
}
#[inline]
pub(crate) fn c_div(a: [f64; 2], b: [f64; 2]) -> Option<[f64; 2]> {
    let scale = b[0].abs().max(b[1].abs());
    if scale == 0.0 || !scale.is_finite() {
        return None;
    }
    let [re, im] = b.map(|x| x / scale);
    let norm = re * re + im * im;
    let [ar, ai] = a.map(|x| x / scale);
    Some([(ar * re + ai * im) / norm, (ai * re - ar * im) / norm])
}
#[inline]
pub(crate) fn c_abs(a: [f64; 2]) -> f64 {
    a[0].hypot(a[1])
}

/// Evaluates polynomial `coeffs_desc[0]*z^n + ... + coeffs_desc[n]` via Horner's method.
pub fn poly_eval_complex(coeffs_desc: &[f64], z: [f64; 2]) -> [f64; 2] {
    let mut acc = [0.0_f64; 2];
    for &c in coeffs_desc {
        acc = c_add(c_mul(acc, z), [c, 0.0]);
    }
    acc
}

pub(crate) fn trim_leading_zeros(coeffs: &[f64]) -> &[f64] {
    let start = coeffs
        .iter()
        .position(|&c| c != 0.0)
        .unwrap_or(coeffs.len());
    &coeffs[start..]
}

/// Pure-Rust Aberth-Ehrlich simultaneous root solver for descending-power real polynomials.
pub fn poly_roots_aberth(coeffs_desc: &[f64]) -> Result<Vec<[f64; 2]>, ControlError> {
    if coeffs_desc.iter().any(|c| !c.is_finite()) {
        return Err(ControlError::NonFinite);
    }
    let c = trim_leading_zeros(coeffs_desc);
    if c.is_empty() {
        return Err(ControlError::InvalidPolynomial);
    }
    let deg = c.len() - 1;
    if deg == 0 {
        return Ok(Vec::new());
    }
    let lead = c[0];
    let norm: Vec<f64> = c.iter().map(|&v| v / lead).collect();
    if deg == 1 {
        return Ok(vec![[-norm[1], 0.0]]);
    }
    if deg == 2 {
        let disc = norm[1] * norm[1] - 4.0 * norm[2];
        if disc >= 0.0 {
            let sd = disc.sqrt();
            return Ok(vec![
                [0.5 * (-norm[1] - sd), 0.0],
                [0.5 * (-norm[1] + sd), 0.0],
            ]);
        }
        let sd = (-disc).sqrt();
        return Ok(vec![
            [0.5 * -norm[1], 0.5 * sd],
            [0.5 * -norm[1], -0.5 * sd],
        ]);
    }

    // Factor out exact zero roots cleanly so multiplicity at 0 never perturbs Aberth.
    let mut zero_roots = 0usize;
    let mut end = norm.len();
    while end > 1 && norm[end - 1] == 0.0 {
        zero_roots += 1;
        end -= 1;
    }
    let active = &norm[..end];
    let n = active.len() - 1;
    let mut roots = Vec::with_capacity(deg);
    roots.resize(zero_roots, [0.0, 0.0]);
    if n == 0 {
        return Ok(roots);
    }

    let dpoly: Vec<f64> = (0..n).map(|i| active[i] * (n - i) as f64).collect();
    let max_coeff = active[1..].iter().map(|v| v.abs()).fold(0.0_f64, f64::max);
    let r0 = (1.0 + max_coeff).powf(1.0 / n as f64).clamp(0.4, 50.0);
    let centroid = -active[1] / n as f64;

    let mut z: Vec<[f64; 2]> = (0..n)
        .map(|k| {
            let angle = std::f64::consts::TAU * (k as f64 + 0.137) / n as f64 + 0.21;
            [centroid + r0 * angle.cos(), r0 * angle.sin()]
        })
        .collect();

    for _ in 0..260 {
        let mut max_step = 0.0_f64;
        for i in 0..n {
            let zi = z[i];
            let p = poly_eval_complex(active, zi);
            let dp = poly_eval_complex(&dpoly, zi);
            let Some(w) = c_div(p, dp) else {
                continue;
            };
            let mut sum = [0.0_f64; 2];
            for (j, &zj) in z.iter().enumerate() {
                if i != j
                    && let Some(term) = c_div([1.0, 0.0], c_sub(zi, zj))
                {
                    sum = c_add(sum, term);
                }
            }
            let denom = c_sub([1.0, 0.0], c_mul(w, sum));
            let step = c_div(w, denom).unwrap_or(w);
            z[i] = c_sub(zi, step);
            max_step = max_step.max(c_abs(step));
        }
        if max_step < 1e-13 {
            break;
        }
    }
    for r in &mut z {
        if r[1].abs() < 1e-10 {
            r[1] = 0.0;
        }
    }
    roots.extend(z);
    Ok(roots)
}

/// Rational transfer function `G(s) = N(s) / D(s) * exp(-tau * s)` in descending powers of `s`.
#[derive(Clone, Debug)]
pub struct TransferFunction {
    num: Arc<[f64]>,
    den: Arc<[f64]>,
    delay_sec: f64,
}

impl TransferFunction {
    pub fn new(num: &[f64], den: &[f64]) -> Result<Self, ControlError> {
        if num.is_empty() || den.is_empty() {
            return Err(ControlError::Empty);
        }
        if num.iter().chain(den.iter()).any(|v| !v.is_finite()) {
            return Err(ControlError::NonFinite);
        }
        let n = trim_leading_zeros(num);
        let d = trim_leading_zeros(den);
        if d.is_empty() {
            return Err(ControlError::InvalidPolynomial);
        }
        let n_slice = if n.is_empty() { &[0.0][..] } else { n };
        Ok(Self {
            num: Arc::from(n_slice),
            den: Arc::from(d),
            delay_sec: 0.0,
        })
    }

    pub fn with_delay(mut self, delay_sec: f64) -> Result<Self, ControlError> {
        if !delay_sec.is_finite() || delay_sec < 0.0 {
            return Err(ControlError::NonFinite);
        }
        self.delay_sec = delay_sec;
        Ok(self)
    }

    pub fn num(&self) -> &[f64] {
        &self.num
    }
    pub fn den(&self) -> &[f64] {
        &self.den
    }
    pub fn delay_sec(&self) -> f64 {
        self.delay_sec
    }

    pub fn eval_s(&self, s: [f64; 2]) -> Option<[f64; 2]> {
        let val = c_div(
            poly_eval_complex(&self.num, s),
            poly_eval_complex(&self.den, s),
        )?;
        if self.delay_sec > 0.0 {
            let decay = (-self.delay_sec * s[0]).exp();
            let angle = -self.delay_sec * s[1];
            let (si, co) = angle.sin_cos();
            Some(c_mul(val, [decay * co, decay * si]))
        } else {
            Some(val)
        }
    }

    pub fn eval_jw(&self, omega_rad_s: f64) -> Option<[f64; 2]> {
        omega_rad_s
            .is_finite()
            .then(|| self.eval_s([0.0, omega_rad_s]))
            .flatten()
    }
}
