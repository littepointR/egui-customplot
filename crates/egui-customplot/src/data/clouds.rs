use std::sync::Arc;

use crate::{AxisScale, PlotBounds};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CloudError {
    Empty,
    LengthMismatch,
    NoPositiveWeights,
}

impl std::fmt::Display for CloudError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "invalid word/bubble cloud data: {self:?}")
    }
}
impl std::error::Error for CloudError {}

#[derive(Clone, Debug, PartialEq)]
pub struct PlacedWord {
    pub text: String,
    pub weight: f64,
    pub font_scale: f32,
    pub center: [f64; 2],
    pub half_size: [f64; 2],
    pub color_index: usize,
}

/// Deterministic Archimedean-spiral packed word cloud in normalized coordinates (`ML-029` `wordcloud`).
#[derive(Clone, Debug)]
pub struct WordCloudData {
    words: Arc<[PlacedWord]>,
    bounds: [f64; 4],
}

impl WordCloudData {
    pub fn new(words: &[&str], weights: &[f64]) -> Result<Self, CloudError> {
        if words.is_empty() {
            return Err(CloudError::Empty);
        }
        if words.len() != weights.len() {
            return Err(CloudError::LengthMismatch);
        }
        let mut items: Vec<(usize, &str, f64)> = words
            .iter()
            .zip(weights.iter())
            .enumerate()
            .filter_map(|(i, (&w, &wt))| {
                (!w.trim().is_empty() && wt.is_finite() && wt > 0.0).then_some((i, w, wt))
            })
            .collect();
        if items.is_empty() {
            return Err(CloudError::NoPositiveWeights);
        }
        items.sort_by(|a, b| b.2.total_cmp(&a.2).then_with(|| a.0.cmp(&b.0)));
        items.truncate(256);

        let w_min = items.last().map(|x| x.2).unwrap_or(1.0);
        let w_max = items[0].2;
        let span = (w_max - w_min).max(0.0);

        let mut placed: Vec<PlacedWord> = Vec::with_capacity(items.len());
        let [mut min_x, mut min_y, mut max_x, mut max_y] = [0.0_f64; 4];

        for (rank, (orig_idx, text, weight)) in items.into_iter().enumerate() {
            let t = if span > 0.0 {
                (weight - w_min) / span
            } else {
                0.5
            };
            let scale = 0.55 + 1.15 * t.sqrt();
            let chars = text.chars().count().max(1) as f64;
            let hw = chars * scale * 0.052 + 0.03;
            let hh = scale * 0.085 + 0.018;

            let mut best = [0.0_f64, 0.0_f64];
            if !placed.is_empty() {
                let mut theta = (rank as f64) * 0.7;
                for step in 0..2400 {
                    let r = 0.012 * theta;
                    let cx = r * theta.cos();
                    let cy = r * theta.sin() * 0.72;
                    let overlaps = placed.iter().any(|p| {
                        (cx - p.center[0]).abs() < (hw + p.half_size[0] + 0.012)
                            && (cy - p.center[1]).abs() < (hh + p.half_size[1] + 0.010)
                    });
                    if !overlaps {
                        best = [cx, cy];
                        break;
                    }
                    theta += if step < 400 { 0.14 } else { 0.19 };
                    if step == 2399 {
                        best = [cx, cy];
                    }
                }
            }
            if rank == 0 {
                min_x = best[0] - hw;
                max_x = best[0] + hw;
                min_y = best[1] - hh;
                max_y = best[1] + hh;
            } else {
                min_x = min_x.min(best[0] - hw);
                max_x = max_x.max(best[0] + hw);
                min_y = min_y.min(best[1] - hh);
                max_y = max_y.max(best[1] + hh);
            }
            placed.push(PlacedWord {
                text: text.to_owned(),
                weight,
                font_scale: (scale as f32).clamp(0.45, 2.2),
                center: best,
                half_size: [hw, hh],
                color_index: orig_idx,
            });
        }
        let pad_x = ((max_x - min_x) * 0.08).max(0.08);
        let pad_y = ((max_y - min_y) * 0.08).max(0.08);
        Ok(Self {
            words: placed.into(),
            bounds: [min_x - pad_x, min_y - pad_y, max_x + pad_x, max_y + pad_y],
        })
    }

    pub fn words(&self) -> &[PlacedWord] {
        &self.words
    }

    pub fn bounds(&self, scales: [AxisScale; 2]) -> Option<PlotBounds> {
        if scales != [AxisScale::Linear, AxisScale::Linear] {
            return None;
        }
        PlotBounds::new(
            [self.bounds[0], self.bounds[1]],
            [self.bounds[2], self.bounds[3]],
        )
        .ok()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct PackedBubble {
    pub label: String,
    pub value: f64,
    pub center: [f64; 2],
    pub radius: f64,
    pub group: usize,
}

/// Deterministic force-relaxed circle-packed bubble cloud (`ML-030` `bubblecloud`).
#[derive(Clone, Debug)]
pub struct BubbleCloudData {
    bubbles: Arc<[PackedBubble]>,
    bounds: [f64; 4],
}

impl BubbleCloudData {
    pub fn new(
        labels: &[&str],
        values: &[f64],
        groups: Option<&[usize]>,
    ) -> Result<Self, CloudError> {
        if labels.is_empty() {
            return Err(CloudError::Empty);
        }
        if labels.len() != values.len() || groups.is_some_and(|g| g.len() != labels.len()) {
            return Err(CloudError::LengthMismatch);
        }
        let mut items: Vec<(usize, &str, f64, usize)> = labels
            .iter()
            .zip(values.iter())
            .enumerate()
            .filter_map(|(i, (&lbl, &val))| {
                (val.is_finite() && val > 0.0).then_some((i, lbl, val, groups.map_or(i, |g| g[i])))
            })
            .collect();
        if items.is_empty() {
            return Err(CloudError::NoPositiveWeights);
        }
        items.sort_by(|a, b| b.2.total_cmp(&a.2).then_with(|| a.0.cmp(&b.0)));
        items.truncate(256);

        let v_max = items[0].2;
        let mut bubbles: Vec<PackedBubble> = Vec::with_capacity(items.len());
        for (rank, (_, label, val, group)) in items.into_iter().enumerate() {
            let radius = 0.08 + 0.34 * (val / v_max).sqrt();
            let center = if rank == 0 {
                [0.0, 0.0]
            } else {
                let mut angle = rank as f64 * 2.399963229728653; // golden angle
                let mut r_orbit = bubbles[0].radius + radius + 0.02;
                let mut pos = [r_orbit * angle.cos(), r_orbit * angle.sin()];
                for _ in 0..2000 {
                    let hit = bubbles.iter().any(|b| {
                        let dx = pos[0] - b.center[0];
                        let dy = pos[1] - b.center[1];
                        let min_d = b.radius + radius + 0.012;
                        dx * dx + dy * dy < min_d * min_d
                    });
                    if !hit {
                        break;
                    }
                    angle += 0.35;
                    r_orbit += 0.012;
                    pos = [r_orbit * angle.cos(), r_orbit * angle.sin()];
                }
                pos
            };
            bubbles.push(PackedBubble {
                label: label.to_owned(),
                value: val,
                center,
                radius,
                group,
            });
        }

        // Deterministic relaxation toward origin followed by strict overlap removal.
        let n = bubbles.len();
        for pass in 0..48 {
            if pass < 16 {
                for b in &mut bubbles {
                    b.center[0] *= 0.94;
                    b.center[1] *= 0.94;
                }
            }
            let mut moved = false;
            for i in 0..n {
                for j in (i + 1)..n {
                    let dx = bubbles[j].center[0] - bubbles[i].center[0];
                    let dy = bubbles[j].center[1] - bubbles[i].center[1];
                    let min_d = bubbles[i].radius + bubbles[j].radius + 0.012;
                    let d2 = dx * dx + dy * dy;
                    if d2 < min_d * min_d {
                        moved = true;
                        let (nx, ny, d) = if d2 <= 1e-12 {
                            (1.0, 0.0, 0.0)
                        } else {
                            let d = d2.sqrt();
                            (dx / d, dy / d, d)
                        };
                        let push = 0.5 * (min_d - d);
                        bubbles[i].center[0] -= nx * push;
                        bubbles[i].center[1] -= ny * push;
                        bubbles[j].center[0] += nx * push;
                        bubbles[j].center[1] += ny * push;
                    }
                }
            }
            if pass >= 16 && !moved {
                break;
            }
        }
        let mut lo = [f64::INFINITY; 2];
        let mut hi = [f64::NEG_INFINITY; 2];
        for b in &bubbles {
            lo[0] = lo[0].min(b.center[0] - b.radius);
            lo[1] = lo[1].min(b.center[1] - b.radius);
            hi[0] = hi[0].max(b.center[0] + b.radius);
            hi[1] = hi[1].max(b.center[1] + b.radius);
        }
        let pad = 0.08;
        Ok(Self {
            bubbles: bubbles.into(),
            bounds: [lo[0] - pad, lo[1] - pad, hi[0] + pad, hi[1] + pad],
        })
    }

    pub fn bubbles(&self) -> &[PackedBubble] {
        &self.bubbles
    }

    pub fn bounds(&self, scales: [AxisScale; 2]) -> Option<PlotBounds> {
        if scales != [AxisScale::Linear, AxisScale::Linear] {
            return None;
        }
        PlotBounds::new(
            [self.bounds[0], self.bounds[1]],
            [self.bounds[2], self.bounds[3]],
        )
        .ok()
    }
}
