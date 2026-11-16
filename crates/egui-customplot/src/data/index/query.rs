use std::ops::Range;

use crate::{AxisScale, PlotPoint, PlotTransform};

use super::{LineIndex, LineQuery, QueryStats};

impl LineIndex {
    /// Conservatively include a neighbor at each boundary, also for picking:
    /// final screen-distance filtering decides the actual hit.
    pub(crate) fn candidates(
        &self,
        points: &[PlotPoint],
        scales: [AxisScale; 2],
        x: [f64; 2],
        stats: &mut QueryStats,
    ) -> Vec<Range<usize>> {
        self.runs(scales)
            .iter()
            .filter_map(|run| {
                stats.visited_runs += 1;
                let slice = &points[run.clone()];
                if slice.last().unwrap()[0] < x[0] || slice[0][0] > x[1] {
                    return None;
                }
                let start = slice
                    .partition_point(|p| {
                        stats.comparisons += 1;
                        p[0] < x[0]
                    })
                    .saturating_sub(1);
                let end = (slice.partition_point(|p| {
                    stats.comparisons += 1;
                    p[0] <= x[1]
                }) + 1)
                    .min(slice.len());
                Some((run.start + start)..(run.start + end))
            })
            .collect()
    }

    pub(crate) fn query(
        &self,
        points: &[PlotPoint],
        transform: PlotTransform,
        pixels_per_point: f32,
        reduce: bool,
    ) -> LineQuery {
        let mut stats = QueryStats::default();
        let x = [transform.bounds().min()[0], transform.bounds().max()[0]];
        let ranges = self.candidates(points, transform.scales(), x, &mut stats);
        let candidate_bytes = ranges.capacity() * std::mem::size_of::<Range<usize>>();
        // Cap the column count, not the sample count. Dense break-heavy data can
        // still emit many points; that cost is reported, never silently hidden.
        let columns = (transform.rect().width() * pixels_per_point.max(0.01))
            .ceil()
            .clamp(1.0, 16_384.0) as usize;
        let scale = transform.scales()[0];
        let lo = scale.forward(x[0]).expect("valid transform");
        let hi = scale.forward(x[1]).expect("valid transform");
        let mut paths = Vec::new();
        for range in ranges {
            if range.len() < 2 {
                continue;
            }
            let selected = if !reduce || range.len() <= columns * 4 {
                range.collect()
            } else {
                stats.reduced = true;
                let mut selected = Vec::with_capacity((columns * 4).min(range.len()));
                let (bx0, bx1) = (x[0].min(x[1]), x[0].max(x[1]));
                let v0 = scale
                    .forward(points[range.start][0].clamp(bx0, bx1))
                    .expect("valid transform");
                let v1 = scale
                    .forward(points[range.end - 1][0].clamp(bx0, bx1))
                    .expect("valid transform");
                let (first_col, last_col) = if (hi - lo).abs() > 0.0 {
                    let u0 = ((v0 - lo) / (hi - lo)).clamp(0.0, 1.0);
                    let u1 = ((v1 - lo) / (hi - lo)).clamp(0.0, 1.0);
                    let f = ((u0.min(u1) * columns as f64).floor() as usize).clamp(1, columns);
                    let l = ((u0.max(u1) * columns as f64).ceil() as usize + 1).clamp(f, columns);
                    (f, l)
                } else {
                    (1, columns)
                };
                let mut start = range.start;
                for column in first_col..=last_col {
                    let end = if column == last_col {
                        range.end
                    } else {
                        let boundary = scale
                            .inverse(lo + (hi - lo) * column as f64 / columns as f64)
                            .unwrap_or(x[1]);
                        let remaining_cols = (last_col - column + 1).max(1);
                        let hint = ((range.end - start) / remaining_cols).max(1);
                        gallop_partition_point(
                            points,
                            start,
                            range.end,
                            hint,
                            boundary,
                            &mut stats.comparisons,
                        )
                    };
                    if end > start {
                        let extrema = self.extrema(points, start..end, &mut stats);
                        let mut indices =
                            [start, extrema.min as usize, extrema.max as usize, end - 1];
                        indices.sort_unstable();
                        for index in indices {
                            if selected.last() != Some(&index) {
                                selected.push(index);
                            }
                        }
                    }
                    start = end;
                    if start >= range.end {
                        break;
                    }
                }
                selected
            };
            stats.selected_points += selected.len();
            paths.push(selected);
        }
        let capacity_bytes = candidate_bytes
            + paths.capacity() * std::mem::size_of::<Vec<usize>>()
            + paths
                .iter()
                .map(|p| p.capacity() * std::mem::size_of::<usize>())
                .sum::<usize>();
        LineQuery {
            paths,
            stats,
            capacity_bytes,
        }
    }
}

/// Hint-assisted exponential galloping search returning the first index in
/// `start..end` where `points[idx][0] >= boundary`, identical to `partition_point`.
pub(crate) fn gallop_partition_point(
    points: &[PlotPoint],
    start: usize,
    end: usize,
    hint: usize,
    boundary: f64,
    comparisons: &mut usize,
) -> usize {
    if start >= end {
        return start;
    }
    if end - start <= 64 {
        return start
            + points[start..end].partition_point(|p| {
                *comparisons += 1;
                p[0] < boundary
            });
    }
    *comparisons += 1;
    if points[start][0] >= boundary {
        return start;
    }
    let mut lo = start + 1;
    let mut hi = end;
    if hint > 64 && start + hint < end {
        let margin = (hint >> 3) + 16;
        let h_lo = start + hint.saturating_sub(margin);
        let h_hi = (start + hint + margin).min(end - 1);
        *comparisons += 1;
        if points[h_lo][0] < boundary {
            lo = h_lo + 1;
            *comparisons += 1;
            if points[h_hi][0] >= boundary {
                hi = h_hi;
                return lo
                    + points[lo..hi].partition_point(|p| {
                        *comparisons += 1;
                        p[0] < boundary
                    });
            }
            lo = h_hi + 1;
            let mut step = margin.max(64);
            while lo < hi {
                let probe = (lo + step).min(hi - 1);
                *comparisons += 1;
                if points[probe][0] < boundary {
                    lo = probe + 1;
                    step = step.saturating_mul(2);
                } else {
                    hi = probe;
                    break;
                }
            }
            return lo
                + points[lo..hi].partition_point(|p| {
                    *comparisons += 1;
                    p[0] < boundary
                });
        } else {
            hi = h_lo;
        }
    }
    let mut step = 1usize;
    while lo < hi {
        let probe = (lo + step).min(hi - 1);
        *comparisons += 1;
        if points[probe][0] < boundary {
            lo = probe + 1;
            step = step.saturating_mul(2);
        } else {
            hi = probe;
            break;
        }
    }
    lo + points[lo..hi].partition_point(|p| {
        *comparisons += 1;
        p[0] < boundary
    })
}
