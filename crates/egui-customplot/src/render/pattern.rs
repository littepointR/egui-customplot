use egui::Pos2;

use crate::LinePattern;

const MAX_DASHES: usize = 100_000;

pub(super) struct PreparedPattern {
    pub paths: Vec<Vec<Pos2>>,
    pub truncated: bool,
}

/// Dash the already clipped reference path. Preserve phase across vertices, but
/// never across separate paths (NaN/log-domain breaks or offscreen excursions).
pub(super) fn apply_pattern(paths: Vec<Vec<Pos2>>, pattern: LinePattern) -> PreparedPattern {
    let LinePattern::Dashed { length, gap } = pattern else {
        return PreparedPattern {
            paths,
            truncated: false,
        };
    };
    debug_assert!(pattern.is_valid());
    let mut result = PreparedPattern {
        paths: Vec::new(),
        truncated: false,
    };
    for path in paths {
        let mut on = true;
        let mut remaining = f64::from(length);
        let mut dash = Vec::new();
        for pair in path.windows(2) {
            let delta = pair[1] - pair[0];
            let distance = f64::from(delta.x).hypot(f64::from(delta.y));
            if distance == 0.0 {
                continue;
            }
            let mut progress = 0.0;
            while progress < distance {
                let step = remaining.min(distance - progress);
                let next = progress + step;
                if next <= progress || !next.is_finite() || result.paths.len() >= MAX_DASHES {
                    result.truncated = true;
                    return result;
                }
                let at = |offset: f64| pair[0] + delta * (offset / distance) as f32;
                if on {
                    let start = at(progress);
                    if dash.last() != Some(&start) {
                        dash.push(start);
                    }
                    dash.push(at(next));
                }
                progress = next;
                remaining -= step;
                if remaining <= 0.0 {
                    if on && dash.len() >= 2 {
                        result.paths.push(std::mem::take(&mut dash));
                    }
                    on = !on;
                    remaining = f64::from(if on { length } else { gap });
                }
            }
        }
        if dash.len() >= 2 {
            if result.paths.len() == MAX_DASHES {
                result.truncated = true;
                return result;
            }
            result.paths.push(dash);
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dashed(paths: Vec<Vec<Pos2>>) -> PreparedPattern {
        apply_pattern(
            paths,
            LinePattern::Dashed {
                length: 10.0,
                gap: 5.0,
            },
        )
    }

    #[test]
    fn dash_phase_continues_across_vertices_but_resets_at_breaks() {
        let path = vec![
            egui::pos2(0.0, 0.0),
            egui::pos2(5.0, 0.0),
            egui::pos2(30.0, 0.0),
        ];
        let p = dashed(vec![path]);
        assert_eq!(
            p.paths,
            vec![
                vec![
                    egui::pos2(0.0, 0.0),
                    egui::pos2(5.0, 0.0),
                    egui::pos2(10.0, 0.0)
                ],
                vec![egui::pos2(15.0, 0.0), egui::pos2(25.0, 0.0)],
            ]
        );
        let p = dashed(vec![
            vec![egui::pos2(0.0, 0.0), egui::pos2(8.0, 0.0)],
            vec![egui::pos2(20.0, 0.0), egui::pos2(28.0, 0.0)],
        ]);
        assert_eq!(p.paths.len(), 2);
        assert_eq!(p.paths[1][0], egui::pos2(20.0, 0.0));
    }

    #[test]
    fn zero_length_segments_are_safe_and_work_is_bounded() {
        assert!(dashed(vec![vec![Pos2::ZERO; 3]]).paths.is_empty());
        let p = apply_pattern(
            vec![vec![Pos2::ZERO, egui::pos2(200_000.0, 0.0)]],
            LinePattern::Dashed {
                length: 0.5,
                gap: 0.5,
            },
        );
        assert!(p.truncated);
        assert_eq!(p.paths.len(), MAX_DASHES);
    }
}
