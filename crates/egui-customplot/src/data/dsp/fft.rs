use std::f64::consts::TAU;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum WindowKind {
    Rectangular,
    #[default]
    Hann,
    Hamming,
    Blackman,
    FlatTop,
}

impl WindowKind {
    pub fn weights(self, len: usize) -> Vec<f64> {
        if len == 0 {
            return Vec::new();
        }
        if len == 1 || self == Self::Rectangular {
            return vec![1.0; len];
        }
        let denom = (len - 1) as f64;
        (0..len)
            .map(|k| {
                let p = TAU * (k as f64 / denom);
                match self {
                    Self::Rectangular => 1.0,
                    Self::Hann => 0.5 * (1.0 - p.cos()),
                    Self::Hamming => 0.54 - 0.46 * p.cos(),
                    Self::Blackman => 0.42 - 0.5 * p.cos() + 0.08 * (2.0 * p).cos(),
                    Self::FlatTop => {
                        0.215_578_95 - 0.416_631_58 * p.cos() + 0.277_263_16 * (2.0 * p).cos()
                            - 0.083_578_95 * (3.0 * p).cos()
                            + 0.006_947_37 * (4.0 * p).cos()
                    }
                }
            })
            .collect()
    }
}

/// In-place iterative Cooley-Tukey Radix-2 FFT/IFFT (`buf.len()` must be a power of 2).
pub fn fft_radix2(buf: &mut [[f64; 2]], inverse: bool) {
    let n = buf.len();
    assert!(
        n.is_power_of_two(),
        "fft_radix2 requires power-of-two length"
    );
    if n <= 1 {
        return;
    }

    let mut j = 0usize;
    for i in 1..n {
        let mut bit = n >> 1;
        while j & bit != 0 {
            j ^= bit;
            bit >>= 1;
        }
        j ^= bit;
        if i < j {
            buf.swap(i, j);
        }
    }

    let sign = if inverse { 1.0 } else { -1.0 };
    let mut len = 2usize;
    while len <= n {
        let half = len >> 1;
        let ang = sign * TAU / len as f64;
        let (w_si, w_co) = ang.sin_cos();
        for start in (0..n).step_by(len) {
            let mut wr = 1.0_f64;
            let mut wi = 0.0_f64;
            for k in 0..half {
                let u = buf[start + k];
                let v0 = buf[start + k + half];
                let vr = wr * v0[0] - wi * v0[1];
                let vi = wr * v0[1] + wi * v0[0];
                buf[start + k] = [u[0] + vr, u[1] + vi];
                buf[start + k + half] = [u[0] - vr, u[1] - vi];
                let nwr = wr * w_co - wi * w_si;
                wi = wr * w_si + wi * w_co;
                wr = nwr;
            }
        }
        len <<= 1;
    }

    if inverse {
        let inv = 1.0 / n as f64;
        for c in buf {
            c[0] *= inv;
            c[1] *= inv;
        }
    }
}

/// Arbitrary-length O(N log N) FFT/IFFT via power-of-two direct Radix-2 or Bluestein Chirp-Z.
pub fn fft_arbitrary(input: &[[f64; 2]], inverse: bool) -> Vec<[f64; 2]> {
    let n = input.len();
    if n <= 1 {
        return input.to_vec();
    }
    if n.is_power_of_two() {
        let mut buf = input.to_vec();
        fft_radix2(&mut buf, inverse);
        return buf;
    }

    let m = (2 * n - 1).next_power_of_two();
    let sign = if inverse { 1.0 } else { -1.0 };
    let mut chirp = Vec::with_capacity(n);
    for k in 0..n {
        let ang = sign * std::f64::consts::PI * ((k as f64 * k as f64) / n as f64);
        let (s, c) = ang.sin_cos();
        chirp.push([c, s]);
    }

    let mut a = vec![[0.0_f64; 2]; m];
    let mut b = vec![[0.0_f64; 2]; m];
    for k in 0..n {
        let [xr, xi] = input[k];
        let [cr, ci] = chirp[k];
        a[k] = [xr * cr - xi * ci, xr * ci + xi * cr];
        b[k] = [cr, -ci];
        if k > 0 {
            b[m - k] = [cr, -ci];
        }
    }

    fft_radix2(&mut a, false);
    fft_radix2(&mut b, false);
    for i in 0..m {
        let [ar, ai] = a[i];
        let [br, bi] = b[i];
        a[i] = [ar * br - ai * bi, ar * bi + ai * br];
    }
    fft_radix2(&mut a, true);

    let scale = if inverse { 1.0 / n as f64 } else { 1.0 };
    (0..n)
        .map(|k| {
            let [yr, yi] = a[k];
            let [cr, ci] = chirp[k];
            [(yr * cr - yi * ci) * scale, (yr * ci + yi * cr) * scale]
        })
        .collect()
}

pub fn fft_real(signal: &[f64]) -> Vec<[f64; 2]> {
    let c: Vec<[f64; 2]> = signal.iter().map(|&r| [r, 0.0]).collect();
    fft_arbitrary(&c, false)
}
