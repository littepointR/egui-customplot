use egui::Pos2;

use crate::{PlotPoint, PlotTransform};

#[derive(Clone, Default)]
pub(super) struct PreparedLine {
    pub paths: Vec<Vec<Pos2>>,
    pub visited_segments: usize,
    pub numeric_rejections: usize,
}

/// Reference O(N) path. Clip in f64 normalized coordinates before casting to f32.
pub(super) fn prepare_line(points: &[PlotPoint], transform: PlotTransform) -> PreparedLine {
    prepare_line_iter(points.iter().copied(), transform)
}

pub(super) fn prepare_line_iter(
    points: impl IntoIterator<Item = PlotPoint>,
    transform: PlotTransform,
) -> PreparedLine {
    let mut prepared = PreparedLine::default();
    let mut path = Vec::new();
    let mut points = points.into_iter();
    let Some(mut previous) = points.next() else {
        return prepared;
    };
    for point in points {
        let pair = [previous, point];
        previous = point;
        prepared.visited_segments += 1;
        if !transform.is_valid_point(pair[0]) || !transform.is_valid_point(pair[1]) {
            flush(&mut path, &mut prepared.paths);
            continue;
        }
        let a = transform.normalized(pair[0]);
        let b = transform.normalized(pair[1]);
        if a.iter().chain(&b).any(|x| !x.is_finite())
            || !(b[0] - a[0]).is_finite()
            || !(b[1] - a[1]).is_finite()
        {
            prepared.numeric_rejections += 1;
            flush(&mut path, &mut prepared.paths);
            continue;
        }
        if let Some((a, b)) = clip_segment(a, b) {
            let a = transform.position_from_normalized(a);
            let b = transform.position_from_normalized(b);
            if path.last() != Some(&a) {
                flush(&mut path, &mut prepared.paths);
                path.push(a);
            }
            path.push(b);
        } else {
            flush(&mut path, &mut prepared.paths);
        }
    }
    flush(&mut path, &mut prepared.paths);
    prepared
}

fn flush(path: &mut Vec<Pos2>, paths: &mut Vec<Vec<Pos2>>) {
    if path.len() >= 2 {
        paths.push(std::mem::take(path));
    } else {
        path.clear();
    }
}

fn clip_segment(a: PlotPoint, b: PlotPoint) -> Option<(PlotPoint, PlotPoint)> {
    let d = [b[0] - a[0], b[1] - a[1]];
    let mut enter: f64 = 0.0;
    let mut exit: f64 = 1.0;
    for (p, q) in [
        (-d[0], a[0]),
        (d[0], 1.0 - a[0]),
        (-d[1], a[1]),
        (d[1], 1.0 - a[1]),
    ] {
        if p == 0.0 {
            if q < 0.0 {
                return None;
            }
        } else {
            let ratio = q / p;
            if p < 0.0 {
                enter = enter.max(ratio);
            } else {
                exit = exit.min(ratio);
            }
            if enter > exit {
                return None;
            }
        }
    }
    let at = |t: f64| {
        [
            (a[0] + t * d[0]).clamp(0.0, 1.0),
            (a[1] + t * d[1]).clamp(0.0, 1.0),
        ]
    };
    Some((at(enter), at(exit)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{AxisScale, PlotBounds};

    fn transform() -> PlotTransform {
        PlotTransform::new(
            PlotBounds::new([0.0; 2], [1.0; 2]).unwrap(),
            egui::Rect::from_min_max(Pos2::ZERO, egui::pos2(100.0, 100.0)),
        )
        .unwrap()
    }

    #[test]
    fn clips_crossing_segment_with_both_endpoints_outside() {
        let p = prepare_line(&[[-1.0, 0.5], [2.0, 0.5]], transform());
        assert_eq!(
            p.paths,
            vec![vec![egui::pos2(0.0, 50.0), egui::pos2(100.0, 50.0)]]
        );
    }

    #[test]
    fn never_bridges_nan_inf_or_offscreen_excursions() {
        for gap in [[f64::NAN; 2], [f64::INFINITY, 0.5], [2.0, 2.0]] {
            let p = prepare_line(
                &[[0.0; 2], [0.5; 2], gap, [0.5, 0.0], [1.0, 0.0]],
                transform(),
            );
            assert_eq!(p.paths.len(), 2);
            assert_eq!(p.visited_segments, 4);
            for point in p.paths.iter().flatten() {
                assert!(transform().rect().contains(*point));
            }
        }
    }

    #[test]
    fn empty_singleton_and_invisible_lines_emit_no_paths() {
        for points in [vec![], vec![[0.0; 2]], vec![[-2.0; 2], [-1.0; 2]]] {
            assert!(prepare_line(&points, transform()).paths.is_empty());
        }
    }

    #[test]
    fn nonpositive_log_samples_break_paths_instead_of_connecting_neighbors() {
        let t = PlotTransform::with_axes(
            PlotBounds::new([1.0; 2], [100.0; 2]).unwrap(),
            transform().rect(),
            [AxisScale::Log10; 2],
            [false; 2],
        )
        .unwrap();
        for gap in [[0.0, 10.0], [10.0, -1.0], [f64::NAN; 2]] {
            let p = prepare_line(&[[1.0; 2], [10.0; 2], gap, [10.0, 1.0], [100.0, 1.0]], t);
            assert_eq!(p.paths.len(), 2);
            assert_eq!(p.visited_segments, 4);
            assert_eq!(p.numeric_rejections, 0);
        }
    }
}
