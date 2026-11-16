use egui::Pos2;

use super::{GeoBounds, GeoPoint, GeoTransform, shortest_dlon};
use crate::axis::clip_normalized_segment;

/// Projects a polyline with antimeridian splitting and segment clipping to `transform.viewport()`.
/// Breaks chains at non-finite / out-of-domain points and splits segments crossing the seam
/// opposite `transform.bounds().center_lon` (`rel_lon == ±180°`).
pub(crate) fn project_geo_polyline(
    points: &[GeoPoint],
    transform: GeoTransform,
    dateline_splits: &mut usize,
) -> Vec<Vec<Pos2>> {
    let mut chains: Vec<Vec<[f64; 2]>> = Vec::new();
    let mut current: Vec<[f64; 2]> = Vec::new();
    let center = transform.bounds().center_lon;
    let half_span = transform.bounds().lon_span * 0.5;
    let y_min = transform
        .projection()
        .lat_to_norm_y(transform.bounds().lat_max);
    let y_max = transform
        .projection()
        .lat_to_norm_y(transform.bounds().lat_min);

    let to_norm = |lat: f64, rel_lon: f64| -> [f64; 2] {
        let u = (rel_lon + half_span) / transform.bounds().lon_span.max(1e-9);
        let wy = transform.projection().lat_to_norm_y(lat);
        let v = 1.0 - (wy - y_min) / (y_max - y_min).max(1e-9);
        [u, v]
    };

    let mut prev_pt: Option<GeoPoint> = None;
    for &pt in points {
        if !pt.is_valid_domain() {
            if current.len() >= 2 {
                chains.push(std::mem::take(&mut current));
            } else {
                current.clear();
            }
            prev_pt = None;
            continue;
        }
        let rel = shortest_dlon(center, pt.lon);
        if let Some(p0) = prev_pt {
            let rel0 = shortest_dlon(center, p0.lon);
            let dlon = shortest_dlon(p0.lon, pt.lon);
            let sum = rel0 + dlon;
            if sum > 180.0 || sum < -180.0 {
                let seam = if sum > 180.0 { 180.0 } else { -180.0 };
                let t = if dlon.abs() > 1e-12 {
                    ((seam - rel0) / dlon).clamp(0.0, 1.0)
                } else {
                    0.5
                };
                let cross_lat = p0.lat + t * (pt.lat - p0.lat);
                current.push(to_norm(cross_lat, seam));
                if current.len() >= 2 {
                    chains.push(std::mem::take(&mut current));
                } else {
                    current.clear();
                }
                *dateline_splits += 1;
                current.push(to_norm(cross_lat, -seam));
            }
        }
        current.push(to_norm(pt.lat, rel));
        prev_pt = Some(pt);
    }
    if current.len() >= 2 {
        chains.push(current);
    }

    let mut out = Vec::new();
    for chain in chains {
        let mut active: Vec<Pos2> = Vec::new();
        for pair in chain.windows(2) {
            if let Some((a, b)) = clip_normalized_segment(pair[0], pair[1]) {
                let pa = transform.pos_from_norm(a);
                let pb = transform.pos_from_norm(b);
                if active.last().is_none_or(|last| last.distance(pa) > 1e-2) {
                    if active.len() >= 2 {
                        out.push(std::mem::take(&mut active));
                    } else {
                        active.clear();
                    }
                    active.push(pa);
                }
                active.push(pb);
            } else if active.len() >= 2 {
                out.push(std::mem::take(&mut active));
            } else {
                active.clear();
            }
        }
        if active.len() >= 2 {
            out.push(active);
        }
    }
    out
}

/// Returns `(lat_ticks, rel_lon_ticks)` where `rel_lon` is offset from `bounds.center_lon`.
pub(crate) fn geo_graticule_ticks(bounds: GeoBounds, max_ticks: usize) -> (Vec<f64>, Vec<f64>) {
    let lat_ticks = nice_degrees(bounds.lat_min, bounds.lat_max, max_ticks.max(2));
    let rel_ticks = nice_degrees(
        -0.5 * bounds.lon_span,
        0.5 * bounds.lon_span,
        max_ticks.max(2),
    );
    (lat_ticks, rel_ticks)
}

fn nice_degrees(lo: f64, hi: f64, max_ticks: usize) -> Vec<f64> {
    let span = hi - lo;
    if !span.is_finite() || span <= 0.0 {
        return Vec::new();
    }
    let steps = [
        0.05, 0.1, 0.2, 0.5, 1.0, 2.0, 5.0, 10.0, 15.0, 20.0, 30.0, 45.0, 60.0, 90.0,
    ];
    let step = steps
        .iter()
        .copied()
        .find(|&s| (span / s).floor() as usize <= max_ticks)
        .unwrap_or(90.0);
    let first = (lo / step).ceil() * step;
    let mut out = Vec::new();
    let mut k = 0;
    loop {
        let v = first + (k as f64) * step;
        if v > hi + 1e-9 || out.len() >= 24 {
            break;
        }
        if v >= lo - 1e-9 {
            out.push(v);
        }
        k += 1;
    }
    out
}

pub(crate) fn format_lat(lat: f64) -> String {
    let hemi = if lat > 1e-6 {
        "N"
    } else if lat < -1e-6 {
        "S"
    } else {
        ""
    };
    let mag = lat.abs();
    if (mag - mag.round()).abs() < 1e-3 {
        format!("{:.0}°{hemi}", mag.round())
    } else {
        format!("{mag:.1}°{hemi}")
    }
}

pub(crate) fn format_lon(lon: f64) -> String {
    let w = super::wrap_lon_180(lon);
    let mag = w.abs();
    let hemi = if mag <= 1e-6 || (mag - 180.0).abs() <= 1e-6 {
        ""
    } else if w > 0.0 {
        "E"
    } else {
        "W"
    };
    if (mag - mag.round()).abs() < 1e-3 {
        format!("{:.0}°{hemi}", mag.round())
    } else {
        format!("{mag:.1}°{hemi}")
    }
}
