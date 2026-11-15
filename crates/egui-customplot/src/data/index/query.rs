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
                let mut selected = Vec::new();
                let mut start = range.start;
                for column in 1..=columns {
                    let end = if column == columns {
                        range.end
                    } else {
                        let boundary = scale
                            .inverse(lo + (hi - lo) * column as f64 / columns as f64)
                            .unwrap_or(x[1]);
                        start
                            + points[start..range.end].partition_point(|p| {
                                stats.comparisons += 1;
                                p[0] < boundary
                            })
                    };
                    if end > start {
                        let extrema = self.extrema(points, start..end, &mut stats);
                        let mut indices = [start, extrema.min, extrema.max, end - 1];
                        indices.sort_unstable();
                        for index in indices {
                            if selected.last() != Some(&index) {
                                selected.push(index);
                            }
                        }
                    }
                    start = end;
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
