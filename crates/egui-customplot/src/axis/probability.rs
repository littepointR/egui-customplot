use super::{AxisScale, AxisTick, ticks::format_tick};

const P_MIN: f64 = 1e-9;
const P_MAX: f64 = 1.0 - 1e-9;

/// Peter J. Acklam inverse standard normal CDF (`Phi^{-1}(p)`), relative error < 1.15e-9.
pub fn norm_inv(p: f64) -> Option<f64> {
    if !p.is_finite() || p <= P_MIN || p >= P_MAX {
        return None;
    }
    const A: [f64; 6] = [
        -3.969_683_028_665_376e1,
        2.209_460_984_245_205e2,
        -2.759_285_104_469_687e2,
        1.383_577_518_672_69e2,
        -3.066_479_806_614_716e1,
        2.506_628_277_459_239e0,
    ];
    const B: [f64; 5] = [
        -5.447_609_879_822_406e1,
        1.615_858_368_580_409e2,
        -1.556_989_798_598_866e2,
        6.680_131_188_771_972e1,
        -1.328_068_155_288_572e1,
    ];
    const C: [f64; 6] = [
        -7.784_894_002_430_293e-3,
        -3.223_964_580_411_365e-1,
        -2.400_758_277_161_838e0,
        -2.549_732_539_343_734e0,
        4.374_664_141_464_968e0,
        2.938_163_982_698_783e0,
    ];
    const D: [f64; 4] = [
        7.784_695_709_041_462e-3,
        3.224_671_290_700_398e-1,
        2.445_134_137_142_996e0,
        3.754_408_661_907_416e0,
    ];
    const P_LOW: f64 = 0.02425;
    const P_HIGH: f64 = 1.0 - P_LOW;

    if p < P_LOW {
        let q = (-2.0 * p.ln()).sqrt();
        let num = (((((C[0] * q + C[1]) * q + C[2]) * q + C[3]) * q + C[4]) * q) + C[5];
        let den = ((((D[0] * q + D[1]) * q + D[2]) * q + D[3]) * q) + 1.0;
        Some(num / den)
    } else if p <= P_HIGH {
        let q = p - 0.5;
        let r = q * q;
        let num = (((((A[0] * r + A[1]) * r + A[2]) * r + A[3]) * r + A[4]) * r + A[5]) * q;
        let den = (((((B[0] * r + B[1]) * r + B[2]) * r + B[3]) * r + B[4]) * r) + 1.0;
        Some(num / den)
    } else {
        let q = (-2.0 * (1.0 - p).ln()).sqrt();
        let num = (((((C[0] * q + C[1]) * q + C[2]) * q + C[3]) * q + C[4]) * q) + C[5];
        let den = ((((D[0] * q + D[1]) * q + D[2]) * q + D[3]) * q) + 1.0;
        Some(-(num / den))
    }
}

/// Standard normal CDF (`Phi(z)`) using Cody's relative-accurate `erfc` approximation.
pub fn norm_cdf(z: f64) -> f64 {
    if !z.is_finite() {
        return if z.is_sign_negative() { 0.0 } else { 1.0 };
    }
    let u = z * std::f64::consts::FRAC_1_SQRT_2;
    let abs_u = u.abs();
    let tail = 0.5 * erfc_pos(abs_u);
    if u >= 0.0 {
        (1.0 - tail).clamp(0.0, 1.0)
    } else {
        tail.clamp(0.0, 1.0)
    }
}

pub(crate) fn norm_cdf_clamped(z: f64) -> Option<f64> {
    z.is_finite()
        .then(|| norm_cdf(z))
        .filter(|&p| p > P_MIN && p < P_MAX)
}

pub fn weibull_forward(p: f64) -> Option<f64> {
    (p.is_finite() && p > P_MIN && p < P_MAX).then(|| (-(1.0 - p).ln()).ln())
}

pub fn weibull_inverse(w: f64) -> Option<f64> {
    w.is_finite()
        .then(|| 1.0 - (-w.exp()).exp())
        .filter(|&p| p > P_MIN && p < P_MAX)
}

pub(super) fn generate_probability_ticks(
    scale: AxisScale,
    range: [f64; 2],
    pixels: f32,
    tick_labels_visible: bool,
    formatter: Option<&(dyn Fn(f64) -> String + Send + Sync)>,
) -> Vec<AxisTick> {
    const PCT_LEVELS: [f64; 23] = [
        0.0001, 0.001, 0.005, 0.01, 0.02, 0.05, 0.1, 0.2, 0.25, 0.3, 0.4, 0.5, 0.6, 0.7, 0.75, 0.8,
        0.9, 0.95, 0.98, 0.99, 0.995, 0.999, 0.9999,
    ];
    let [min, max] = range;
    let max_ticks = ((pixels / 56.0) as usize).clamp(2, 16);
    let mut in_range: Vec<f64> = PCT_LEVELS
        .into_iter()
        .filter(|&v| v >= min && v <= max && scale.forward(v).is_some())
        .collect();
    if in_range.len() >= 2 {
        let stride = (in_range.len() / max_ticks).max(1);
        let mut ticks = Vec::new();
        for (i, &val) in in_range.iter().enumerate() {
            if i % stride != 0 && i + 1 != in_range.len() {
                continue;
            }
            let label = if !tick_labels_visible {
                String::new()
            } else if let Some(fmt) = formatter {
                fmt(val)
            } else {
                format_prob_label(val)
            };
            ticks.push(AxisTick {
                value: val,
                label,
                major: true,
            });
        }
        return ticks;
    }

    let (Some(lo_t), Some(hi_t)) = (scale.forward(min), scale.forward(max)) else {
        return Vec::new();
    };
    let span_t = hi_t - lo_t;
    if span_t <= 0.0 || !span_t.is_finite() {
        return Vec::new();
    }
    let count = ((pixels / 80.0) as usize).clamp(2, 8);
    let mut ticks = Vec::new();
    for k in 0..count {
        let t = lo_t + span_t * (k as f64 / (count - 1) as f64);
        if let Some(val) = scale.inverse(t) {
            let label = if !tick_labels_visible {
                String::new()
            } else if let Some(fmt) = formatter {
                fmt(val)
            } else {
                format_tick(val, (max - min).abs() * 0.1)
            };
            ticks.push(AxisTick {
                value: val,
                label,
                major: true,
            });
        }
    }
    in_range.clear();
    ticks
}

fn format_prob_label(p: f64) -> String {
    let pct = p * 100.0;
    if (pct - pct.round()).abs() < 1e-6 {
        format!("{:.0}%", pct.round())
    } else if (pct * 10.0 - (pct * 10.0).round()).abs() < 1e-5 {
        format!("{pct:.1}%")
    } else {
        format!("{pct:.2}%")
    }
}

/// W. J. Cody rational approximation to `erfc(x)` for `x >= 0`.
fn erfc_pos(x: f64) -> f64 {
    if x < 0.46875 {
        const A: [f64; 5] = [
            3.161_123_743_870_565_6e00,
            1.138_641_541_510_501_6e02,
            3.774_852_376_853_02e2,
            3.209_377_589_138_469_5e03,
            1.857_777_061_846_031_5e-01,
        ];
        const B: [f64; 4] = [
            2.360_129_095_234_412_1e01,
            2.440_246_379_344_441_7e02,
            1.282_616_526_077_372_3e03,
            2.844_236_833_439_171e3,
        ];
        let xsq = x * x;
        let num = (((A[4] * xsq + A[0]) * xsq + A[1]) * xsq + A[2]) * xsq + A[3];
        let den = (((xsq + B[0]) * xsq + B[1]) * xsq + B[2]) * xsq + B[3];
        1.0 - x * (num / den)
    } else if x < 4.0 {
        const C: [f64; 9] = [
            5.641_884_969_886_701e-1,
            8.883_149_794_388_375,
            6.611_919_063_714_163e1,
            2.986_351_381_974_001e2,
            8.819_522_212_417_69e2,
            1.712_047_612_634_070_6e03,
            2.051_078_377_826_071_5e03,
            1.230_339_354_797_997_2e03,
            2.153_115_354_744_038_5e-08,
        ];
        const D: [f64; 8] = [
            1.574_492_611_070_983_5e01,
            1.176_939_508_913_125e2,
            5.371_811_018_620_099e2,
            1.621_389_574_566_690_2e03,
            3.290_799_235_733_459_6e03,
            4.362_619_090_143_247e3,
            3.439_367_674_143_721_6e03,
            1.230_339_354_803_749_4e03,
        ];
        let mut xnum = C[8] * x;
        let mut xden = x;
        for i in 0..7 {
            xnum = (xnum + C[i]) * x;
            xden = (xden + D[i]) * x;
        }
        let ratio = (xnum + C[7]) / (xden + D[7]);
        (-x * x).exp() * ratio
    } else {
        const P: [f64; 6] = [
            3.053_266_349_612_323_4e-01,
            3.603_448_999_498_044_4e-01,
            1.257_817_261_112_292_5e-01,
            1.608_378_514_874_228e-2,
            6.587_491_615_298_378e-4,
            1.631_538_713_730_209_8e-02,
        ];
        const Q: [f64; 5] = [
            2.568_520_192_289_822,
            1.872_952_849_923_460_5e00,
            5.279_051_029_514_285e-1,
            6.051_834_131_244_132e-2,
            2.335_204_976_268_691_9e-03,
        ];
        let xsq = 1.0 / (x * x);
        let mut xnum = P[5] * xsq;
        let mut xden = xsq;
        for i in 0..4 {
            xnum = (xnum + P[i]) * xsq;
            xden = (xden + Q[i]) * xsq;
        }
        let res = xsq * (xnum + P[4]) / (xden + Q[4]);
        let corr = (std::f64::consts::FRAC_2_SQRT_PI * 0.5 - res) / x;
        (-x * x).exp() * corr
    }
}
