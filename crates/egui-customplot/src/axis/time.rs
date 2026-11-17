use super::{AxisTick, ticks::format_tick};

const SECS_PER_DAY: f64 = 86_400.0;
const CIVIL_RANGE_LIMIT: f64 = 3.15e11;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TimeAxisMode {
    UtcDateTime { offset_seconds: i32 },
    Duration,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CivilDateTime {
    pub year: i32,
    pub month: u8,
    pub day: u8,
    pub hour: u8,
    pub minute: u8,
    pub second: u8,
    pub millis: u16,
}

fn is_leap_year(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

fn days_in_month(year: i32, month: u8) -> Option<u8> {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => Some(31),
        4 | 6 | 9 | 11 => Some(30),
        2 => Some(if is_leap_year(year) { 29 } else { 28 }),
        _ => None,
    }
}

fn civil_to_days(year: i64, month: u8, day: u8) -> i64 {
    let y = year - i64::from(month <= 2);
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = y - era * 400;
    let m = i64::from(month);
    let doy = (153 * (if m > 2 { m - 3 } else { m + 9 }) + 2) / 5 + i64::from(day) - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

fn days_to_civil(z: i64) -> (i32, u8, u8) {
    let z = z + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    ((y + i64::from(m <= 2)) as i32, m as u8, d as u8)
}

pub fn civil_to_unix_secs(
    year: i32,
    month: u8,
    day: u8,
    hour: u8,
    minute: u8,
    second: f64,
) -> Option<f64> {
    let max_day = days_in_month(year, month)?;
    if day == 0 || day > max_day || hour >= 24 || minute >= 60 {
        return None;
    }
    if !second.is_finite() || !(0.0..60.0).contains(&second) {
        return None;
    }
    let days = civil_to_days(i64::from(year), month, day);
    let secs =
        (days as f64) * SECS_PER_DAY + f64::from(hour) * 3600.0 + f64::from(minute) * 60.0 + second;
    secs.is_finite().then_some(secs)
}

pub fn unix_secs_to_civil(unix_secs: f64) -> Option<CivilDateTime> {
    if !unix_secs.is_finite() || !(-CIVIL_RANGE_LIMIT..=CIVIL_RANGE_LIMIT).contains(&unix_secs) {
        return None;
    }
    let total_ms = (unix_secs * 1000.0).round() as i64;
    let total_secs = total_ms.div_euclid(1000);
    let millis = total_ms.rem_euclid(1000) as u16;
    let days = total_secs.div_euclid(86_400);
    let day_secs = total_secs.rem_euclid(86_400);
    let (year, month, day) = days_to_civil(days);
    Some(CivilDateTime {
        year,
        month,
        day,
        hour: (day_secs / 3600) as u8,
        minute: ((day_secs % 3600) / 60) as u8,
        second: (day_secs % 60) as u8,
        millis,
    })
}

pub fn format_duration_hms(seconds: f64, step: f64) -> String {
    if !seconds.is_finite() || !step.is_finite() || step <= 0.0 {
        return format_tick(seconds, step.max(1e-9));
    }
    let neg = seconds < 0.0 && seconds.abs() >= step * 1e-6;
    let sign = if neg { "-" } else { "" };
    let abs_s = if seconds.abs() < step * 1e-6 {
        0.0
    } else {
        seconds.abs()
    };
    if step >= SECS_PER_DAY {
        let days = abs_s / SECS_PER_DAY;
        let step_days = step / SECS_PER_DAY;
        let prec = if step_days >= 1.0 && (days - days.round()).abs() < 1e-6 {
            0
        } else {
            (-step_days.log10().floor()).clamp(0.0, 3.0) as usize
        };
        return format!("{sign}{days:.prec$}d");
    }
    if step < 1.0 && abs_s < 60.0 {
        let prec = (-step.log10().floor()).clamp(1.0, 6.0) as usize;
        return format!("{sign}{abs_s:.prec$}s");
    }
    let total_ms = (abs_s * 1000.0).round() as u64;
    let total_s = total_ms / 1000;
    let ms = (total_ms % 1000) as u16;
    let days = total_s / 86_400;
    let rem = total_s % 86_400;
    let hh = rem / 3600;
    let mm = (rem % 3600) / 60;
    let ss = rem % 60;
    if step < 1.0 {
        return if days > 0 {
            format!("{sign}{days}d {hh:02}:{mm:02}:{ss:02}.{ms:03}")
        } else {
            format!("{sign}{hh:02}:{mm:02}:{ss:02}.{ms:03}")
        };
    }
    if days > 0 {
        if step < 60.0 || ss != 0 {
            format!("{sign}{days}d {hh:02}:{mm:02}:{ss:02}")
        } else {
            format!("{sign}{days}d {hh:02}:{mm:02}")
        }
    } else if step < 60.0 || ss != 0 {
        format!("{sign}{hh:02}:{mm:02}:{ss:02}")
    } else {
        format!("{sign}{hh:02}:{mm:02}")
    }
}

fn decimal_nice_step(raw: f64) -> Option<f64> {
    if !raw.is_finite() || raw <= 0.0 {
        return None;
    }
    let mag = 10.0_f64.powf(raw.log10().floor());
    let frac = raw / mag;
    let nice = [1.0, 2.0, 2.5, 5.0, 10.0]
        .into_iter()
        .find(|n| *n >= frac)
        .unwrap_or(10.0);
    let step = nice * mag;
    (step.is_finite() && step > 0.0).then_some(step)
}

const TIME_STEPS: &[f64] = &[
    0.001, 0.002, 0.005, 0.01, 0.02, 0.05, 0.1, 0.2, 0.5, 1.0, 2.0, 5.0, 10.0, 15.0, 30.0, 60.0,
    120.0, 300.0, 600.0, 900.0, 1800.0, 3600.0, 7200.0, 10800.0, 14400.0, 21600.0, 43200.0,
    86400.0, 172800.0, 604800.0,
];

fn pick_time_step(raw: f64) -> Option<f64> {
    if !raw.is_finite() || raw <= 0.0 {
        return None;
    }
    if raw < 0.001 {
        return decimal_nice_step(raw);
    }
    if let Some(&s) = TIME_STEPS.iter().find(|&&s| s >= raw) {
        return Some(s);
    }
    decimal_nice_step(raw / SECS_PER_DAY).map(|d| d * SECS_PER_DAY)
}

fn format_datetime_label(local_secs: f64, step: f64, crosses_day: bool) -> String {
    let Some(c) = unix_secs_to_civil(local_secs) else {
        return format_tick(local_secs, step);
    };
    if step >= SECS_PER_DAY {
        format!("{:04}-{:02}-{:02}", c.year, c.month, c.day)
    } else if step >= 60.0 && c.second == 0 {
        if crosses_day {
            format!("{:02}-{:02} {:02}:{:02}", c.month, c.day, c.hour, c.minute)
        } else {
            format!("{:02}:{:02}", c.hour, c.minute)
        }
    } else if step >= 1.0 {
        if crosses_day {
            format!(
                "{:02}-{:02} {:02}:{:02}:{:02}",
                c.month, c.day, c.hour, c.minute, c.second
            )
        } else {
            format!("{:02}:{:02}:{:02}", c.hour, c.minute, c.second)
        }
    } else {
        format!(
            "{:02}:{:02}:{:02}.{:03}",
            c.hour, c.minute, c.second, c.millis
        )
    }
}

fn generate_calendar_year_month_ticks(
    local_range: [f64; 2],
    offset: f64,
    count: usize,
    raw: f64,
    range: [f64; 2],
    tick_labels_visible: bool,
    formatter: Option<&(dyn Fn(f64) -> String + Send + Sync)>,
) -> Option<Vec<AxisTick>> {
    let start_c = unix_secs_to_civil(local_range[0])?;
    let end_c = unix_secs_to_civil(local_range[1])?;
    let mut ticks = Vec::new();
    if raw >= 270.0 * SECS_PER_DAY {
        let span_y = (f64::from(end_c.year - start_c.year)).max(1.0);
        let y_step = decimal_nice_step(span_y / (count - 1) as f64)?.max(1.0) as i64;
        let mut y = i64::from(start_c.year).div_euclid(y_step) * y_step;
        while y <= i64::from(end_c.year) && ticks.len() < 128 {
            if let Ok(y32) = i32::try_from(y)
                && let Some(local) = civil_to_unix_secs(y32, 1, 1, 0, 0, 0.0)
            {
                let v = local - offset;
                if v >= range[0] && v <= range[1] {
                    let label = if !tick_labels_visible {
                        String::new()
                    } else if let Some(fmt) = formatter {
                        fmt(v)
                    } else {
                        format!("{y32:04}")
                    };
                    ticks.push(AxisTick {
                        value: v,
                        label,
                        major: true,
                    });
                }
            }
            y = y.saturating_add(y_step);
        }
        return Some(ticks);
    }
    let m_span = f64::from(
        (end_c.year - start_c.year) * 12 + i32::from(end_c.month) - i32::from(start_c.month),
    )
    .max(1.0);
    let raw_m = (m_span / (count - 1) as f64).max(1.0);
    let m_step = [1i32, 2, 3, 6]
        .into_iter()
        .find(|&s| f64::from(s) >= raw_m)
        .unwrap_or(6);
    let start_idx = start_c.year * 12 + i32::from(start_c.month) - 1;
    let end_idx = end_c.year * 12 + i32::from(end_c.month) - 1;
    let mut cur = start_idx;
    while cur <= end_idx && ticks.len() < 128 {
        let y = cur.div_euclid(12);
        let m = (cur.rem_euclid(12) + 1) as u8;
        if let Some(local) = civil_to_unix_secs(y, m, 1, 0, 0, 0.0) {
            let v = local - offset;
            if v >= range[0] && v <= range[1] {
                let label = if !tick_labels_visible {
                    String::new()
                } else if let Some(fmt) = formatter {
                    fmt(v)
                } else {
                    format!("{y:04}-{m:02}")
                };
                ticks.push(AxisTick {
                    value: v,
                    label,
                    major: true,
                });
            }
        }
        cur += m_step;
    }
    Some(ticks)
}

pub(crate) fn generate_time_ticks(
    mode: TimeAxisMode,
    range: [f64; 2],
    count: usize,
    tick_labels_visible: bool,
    formatter: Option<&(dyn Fn(f64) -> String + Send + Sync)>,
) -> Vec<AxisTick> {
    let [min, max] = range;
    let span = max - min;
    if !span.is_finite() || span <= 0.0 {
        return Vec::new();
    }
    let count = count.max(2);
    let raw = (span / (count - 1) as f64).max(f64::from_bits(1));
    match mode {
        TimeAxisMode::Duration => {
            let Some(step) = pick_time_step(raw) else {
                return Vec::new();
            };
            let start = (min / step).ceil() * step;
            let mut ticks = Vec::new();
            let mut prev = f64::NEG_INFINITY;
            for idx in 0..=128 {
                let v = (idx as f64).mul_add(step, start);
                if v > max {
                    break;
                }
                if v < min || !v.is_finite() || v <= prev {
                    continue;
                }
                prev = v;
                let label = if !tick_labels_visible {
                    String::new()
                } else if let Some(fmt) = formatter {
                    fmt(v)
                } else {
                    format_duration_hms(v, step)
                };
                ticks.push(AxisTick {
                    value: v,
                    label,
                    major: true,
                });
            }
            ticks
        }
        TimeAxisMode::UtcDateTime { offset_seconds } => {
            let offset = f64::from(offset_seconds);
            let l_min = min + offset;
            let l_max = max + offset;
            if !(-CIVIL_RANGE_LIMIT..=CIVIL_RANGE_LIMIT).contains(&l_min)
                || !(-CIVIL_RANGE_LIMIT..=CIVIL_RANGE_LIMIT).contains(&l_max)
            {
                return Vec::new();
            }
            if raw >= 20.0 * SECS_PER_DAY
                && let Some(ticks) = generate_calendar_year_month_ticks(
                    [l_min, l_max],
                    offset,
                    count,
                    raw,
                    range,
                    tick_labels_visible,
                    formatter,
                )
            {
                return ticks;
            }
            let Some(step) = pick_time_step(raw) else {
                return Vec::new();
            };
            let crosses_day = unix_secs_to_civil(l_min)
                .zip(unix_secs_to_civil(l_max))
                .is_none_or(|(a, b)| (a.year, a.month, a.day) != (b.year, b.month, b.day));
            let start = (l_min / step).ceil() * step;
            let mut ticks = Vec::new();
            let mut prev = f64::NEG_INFINITY;
            for idx in 0..=128 {
                let local = (idx as f64).mul_add(step, start);
                let v = local - offset;
                if v > max {
                    break;
                }
                if v < min || !v.is_finite() || v <= prev {
                    continue;
                }
                prev = v;
                let label = if !tick_labels_visible {
                    String::new()
                } else if let Some(fmt) = formatter {
                    fmt(v)
                } else if step < 0.001 {
                    format_tick(local, step)
                } else {
                    format_datetime_label(local, step, crosses_day)
                };
                ticks.push(AxisTick {
                    value: v,
                    label,
                    major: true,
                });
            }
            ticks
        }
    }
}
