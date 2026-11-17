use std::{f64::consts::TAU, sync::Arc};

use crate::{AxisScale, PlotBounds, PlotPoint};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ModernError {
    Empty,
    LengthMismatch,
    NonFinite,
    NegativeWeight,
    InvalidNode,
    InvalidHierarchy,
    InvalidInterval,
}

impl std::fmt::Display for ModernError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Empty => f.write_str("modern chart input must not be empty"),
            Self::LengthMismatch => f.write_str("input dimension lengths do not match"),
            Self::NonFinite => f.write_str("modern chart values must be finite"),
            Self::NegativeWeight => {
                f.write_str("weights/values must be strictly positive or non-negative")
            }
            Self::InvalidNode => f.write_str("invalid Sankey node index or self-loop"),
            Self::InvalidHierarchy => f.write_str("invalid sunburst hierarchy root/parent link"),
            Self::InvalidInterval => {
                f.write_str("Gantt task must have start <= end and progress in [0, 1]")
            }
        }
    }
}

impl std::error::Error for ModernError {}

#[derive(Clone, Debug)]
pub struct RadarSeries {
    pub name: String,
    pub raw_values: Arc<[f64]>,
    pub vertices: Arc<[PlotPoint]>,
}

/// Multi-axis spider/radar chart (`RadarChart`, `EXT-26`).
#[derive(Clone, Debug)]
pub struct RadarData {
    axes_labels: Arc<[String]>,
    axis_limits: Arc<[f64]>,
    spoke_dirs: Arc<[PlotPoint]>,
    series: Arc<[RadarSeries]>,
}

impl RadarData {
    pub fn new(
        axes_labels: &[&str],
        axis_limits: &[f64],
        series: &[(&str, &[f64])],
    ) -> Result<Self, ModernError> {
        let m = axes_labels.len();
        if m < 3 || series.is_empty() {
            return Err(ModernError::Empty);
        }
        if axis_limits.len() != m {
            return Err(ModernError::LengthMismatch);
        }
        if axis_limits.iter().any(|&v| !v.is_finite() || v <= 0.0) {
            return Err(ModernError::NegativeWeight);
        }
        let spoke_dirs: Vec<PlotPoint> = (0..m)
            .map(|k| {
                let ang = std::f64::consts::FRAC_PI_2 - TAU * (k as f64 / m as f64);
                [ang.cos(), ang.sin()]
            })
            .collect();

        let mut out_series = Vec::with_capacity(series.len());
        for &(name, vals) in series {
            if vals.len() != m {
                return Err(ModernError::LengthMismatch);
            }
            if vals.iter().any(|&v| !v.is_finite() || v < 0.0) {
                return Err(ModernError::NegativeWeight);
            }
            let vertices: Vec<PlotPoint> = vals
                .iter()
                .enumerate()
                .map(|(k, &v)| {
                    let frac = (v / axis_limits[k]).clamp(0.0, 1.25);
                    [frac * spoke_dirs[k][0], frac * spoke_dirs[k][1]]
                })
                .collect();
            out_series.push(RadarSeries {
                name: name.to_owned(),
                raw_values: Arc::from(vals),
                vertices: vertices.into(),
            });
        }
        Ok(Self {
            axes_labels: axes_labels.iter().map(|s| (*s).to_owned()).collect(),
            axis_limits: Arc::from(axis_limits),
            spoke_dirs: spoke_dirs.into(),
            series: out_series.into(),
        })
    }

    pub fn axes_labels(&self) -> &[String] {
        &self.axes_labels
    }
    pub fn axis_limits(&self) -> &[f64] {
        &self.axis_limits
    }
    pub fn spoke_dirs(&self) -> &[PlotPoint] {
        &self.spoke_dirs
    }
    pub fn series(&self) -> &[RadarSeries] {
        &self.series
    }
    pub fn sample_bytes(&self) -> usize {
        self.series.iter().map(|s| s.vertices.len() * 16).sum()
    }
    pub fn bounds(&self, scales: [AxisScale; 2]) -> Option<PlotBounds> {
        if scales[0] != AxisScale::Linear || scales[1] != AxisScale::Linear {
            return None;
        }
        PlotBounds::new([-1.28, -1.28], [1.28, 1.28]).ok()
    }
}

#[derive(Clone, Debug)]
pub struct SankeyNode {
    pub index: usize,
    pub label: String,
    pub column: usize,
    pub x0: f64,
    pub x1: f64,
    pub y0: f64,
    pub y1: f64,
    pub total_flow: f64,
}

#[derive(Clone, Copy, Debug)]
pub struct SankeyLink {
    pub source: usize,
    pub target: usize,
    pub value: f64,
}

#[derive(Clone, Debug)]
pub struct SankeyRibbon {
    pub index: usize,
    pub source: usize,
    pub target: usize,
    pub value: f64,
    pub top_curve: Vec<PlotPoint>,
    pub bottom_curve: Vec<PlotPoint>,
}

/// Flow-conserving multi-column Sankey diagram (`SankeyChart`, `EXT-26`).
#[derive(Clone, Debug)]
pub struct SankeyData {
    nodes: Arc<[SankeyNode]>,
    ribbons: Arc<[SankeyRibbon]>,
}

impl SankeyData {
    pub fn new(node_labels: &[&str], links: &[(usize, usize, f64)]) -> Result<Self, ModernError> {
        let n = node_labels.len();
        if n == 0 || links.is_empty() {
            return Err(ModernError::Empty);
        }
        let mut adj = vec![Vec::<usize>::new(); n];
        let mut indeg = vec![0usize; n];
        for &(s, t, v) in links {
            if s >= n || t >= n || s == t {
                return Err(ModernError::InvalidNode);
            }
            if !v.is_finite() || v <= 0.0 {
                return Err(ModernError::NegativeWeight);
            }
            adj[s].push(t);
            indeg[t] += 1;
        }

        // Longest-path column assignment (falls back cleanly if graph has back-edges)
        let mut col = vec![0usize; n];
        let mut q: Vec<usize> = (0..n).filter(|&i| indeg[i] == 0).collect();
        let mut rem_in = indeg;
        let mut visited = 0usize;
        let mut head = 0usize;
        while head < q.len() {
            let u = q[head];
            head += 1;
            visited += 1;
            for &v in &adj[u] {
                col[v] = col[v].max(col[u] + 1);
                rem_in[v] -= 1;
                if rem_in[v] == 0 {
                    q.push(v);
                }
            }
        }
        if visited < n {
            return Err(ModernError::InvalidHierarchy);
        }

        let n_cols = col.iter().copied().max().unwrap_or(0) + 1;
        let mut in_sum = vec![0.0_f64; n];
        let mut out_sum = vec![0.0_f64; n];
        for &(s, t, v) in links {
            out_sum[s] += v;
            in_sum[t] += v;
        }
        let node_flow: Vec<f64> = (0..n)
            .map(|i| in_sum[i].max(out_sum[i]).max(1e-9))
            .collect();

        let mut cols: Vec<Vec<usize>> = vec![Vec::new(); n_cols];
        for (i, &c) in col.iter().enumerate() {
            cols[c].push(i);
        }
        let max_col_flow = cols
            .iter()
            .map(|c| c.iter().map(|&i| node_flow[i]).sum::<f64>())
            .fold(0.0_f64, f64::max)
            .max(1e-9);

        let bar_w = 0.032_f64;
        let x_pad = 0.06_f64;
        let usable_h = 0.86_f64;
        let mut nodes: Vec<Option<SankeyNode>> = vec![None; n];
        for (ci, members) in cols.iter().enumerate() {
            let cx = if n_cols <= 1 {
                0.5
            } else {
                x_pad + (1.0 - 2.0 * x_pad - bar_w) * (ci as f64 / (n_cols - 1) as f64)
            };
            let gap = if members.len() > 1 {
                0.08 / (members.len() - 1) as f64
            } else {
                0.0
            };
            let mut y = 0.95;
            for &ni in members {
                let h = (node_flow[ni] / max_col_flow) * usable_h;
                nodes[ni] = Some(SankeyNode {
                    index: ni,
                    label: node_labels[ni].to_owned(),
                    column: ci,
                    x0: cx,
                    x1: cx + bar_w,
                    y0: y - h,
                    y1: y,
                    total_flow: node_flow[ni],
                });
                y -= h + gap;
            }
        }
        let nodes: Vec<SankeyNode> = nodes.into_iter().map(Option::unwrap).collect();

        let mut out_cursor: Vec<f64> = nodes.iter().map(|nd| nd.y1).collect();
        let mut in_cursor: Vec<f64> = nodes.iter().map(|nd| nd.y1).collect();
        let mut ribbons = Vec::with_capacity(links.len());
        for (idx, &(s, t, v)) in links.iter().enumerate() {
            let dh = (v / max_col_flow) * usable_h;
            let sy1 = out_cursor[s];
            let sy0 = sy1 - dh;
            out_cursor[s] = sy0;
            let ty1 = in_cursor[t];
            let ty0 = ty1 - dh;
            in_cursor[t] = ty0;

            let sx = nodes[s].x1;
            let tx = nodes[t].x0;
            let steps = 20usize;
            let mut top_curve = Vec::with_capacity(steps + 1);
            let mut bottom_curve = Vec::with_capacity(steps + 1);
            let mid_x = 0.5 * (sx + tx);
            for k in 0..=steps {
                let u = k as f64 / steps as f64;
                let omt = 1.0 - u;
                let b0 = omt * omt * omt;
                let b1 = 3.0 * omt * omt * u;
                let b2 = 3.0 * omt * u * u;
                let b3 = u * u * u;
                let x = b0 * sx + b1 * mid_x + b2 * mid_x + b3 * tx;
                top_curve.push([x, b0 * sy1 + b1 * sy1 + b2 * ty1 + b3 * ty1]);
                bottom_curve.push([x, b0 * sy0 + b1 * sy0 + b2 * ty0 + b3 * ty0]);
            }
            ribbons.push(SankeyRibbon {
                index: idx,
                source: s,
                target: t,
                value: v,
                top_curve,
                bottom_curve,
            });
        }

        Ok(Self {
            nodes: nodes.into(),
            ribbons: ribbons.into(),
        })
    }

    pub fn nodes(&self) -> &[SankeyNode] {
        &self.nodes
    }
    pub fn ribbons(&self) -> &[SankeyRibbon] {
        &self.ribbons
    }
    pub fn sample_bytes(&self) -> usize {
        self.ribbons.iter().map(|r| r.top_curve.len() * 32).sum()
    }
    pub fn bounds(&self, scales: [AxisScale; 2]) -> Option<PlotBounds> {
        if scales[0] != AxisScale::Linear || scales[1] != AxisScale::Linear {
            return None;
        }
        PlotBounds::new([-0.02, -0.02], [1.02, 1.02]).ok()
    }
}
