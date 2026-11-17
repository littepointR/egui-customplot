use crate::{
    AxisScale, MultivariateError, PlotBounds, PlotData, PlotPoint, QuiverArrow, QuiverData,
};

#[derive(Clone, Debug)]
pub struct BiplotData {
    loadings_raw: Vec<[f64; 2]>,
    scores_raw: Vec<[f64; 2]>,
    loadings_quiver: QuiverData,
    scores: PlotData,
    var_labels: Vec<String>,
    obs_labels: Vec<String>,
    explained_ratio: [f64; 2],
}

impl BiplotData {
    /// Computes 2D PCA biplot using symmetric covariance Jacobi eigensolver,
    /// MATLAB dominant-loading sign convention, and score normalization into the loading box.
    pub fn from_pca(observations: &[&[f64]]) -> Result<Self, MultivariateError> {
        let n_obs = observations.len();
        if n_obs < 2 {
            return Err(MultivariateError::Empty);
        }
        let n_vars = observations[0].len();
        if n_vars < 2 || observations.iter().any(|r| r.len() != n_vars) {
            return Err(MultivariateError::ShapeMismatch);
        }
        if observations
            .iter()
            .flat_map(|r| r.iter())
            .any(|v| !v.is_finite())
        {
            return Err(MultivariateError::NoFiniteValues);
        }

        let mut means = vec![0.0_f64; n_vars];
        for row in observations {
            for (j, &v) in row.iter().enumerate() {
                means[j] += v;
            }
        }
        for m in &mut means {
            *m /= n_obs as f64;
        }
        let centered: Vec<Vec<f64>> = observations
            .iter()
            .map(|r| r.iter().enumerate().map(|(j, &v)| v - means[j]).collect())
            .collect();

        let denom = (n_obs - 1) as f64;
        let mut cov = vec![0.0_f64; n_vars * n_vars];
        for r in &centered {
            for i in 0..n_vars {
                for j in i..n_vars {
                    cov[i * n_vars + j] += r[i] * r[j] / denom;
                }
            }
        }
        for i in 0..n_vars {
            for j in 0..i {
                cov[i * n_vars + j] = cov[j * n_vars + i];
            }
        }

        let (eigvals, eigvecs) = jacobi_symmetric_eigen(n_vars, &mut cov);
        let total_var: f64 = eigvals.iter().map(|v| v.max(0.0)).sum();
        let explained_ratio = if total_var > 1e-12 {
            [
                eigvals[0].max(0.0) / total_var,
                eigvals[1].max(0.0) / total_var,
            ]
        } else {
            [0.5, 0.5]
        };

        let mut col0 = vec![0.0_f64; n_vars];
        let mut col1 = vec![0.0_f64; n_vars];
        for i in 0..n_vars {
            col0[i] = eigvecs[i * n_vars];
            col1[i] = eigvecs[i * n_vars + 1];
        }
        apply_matlab_sign(&mut col0);
        apply_matlab_sign(&mut col1);

        let loadings_raw: Vec<[f64; 2]> = (0..n_vars).map(|i| [col0[i], col1[i]]).collect();
        let scores_raw: Vec<[f64; 2]> = centered
            .iter()
            .map(|row| {
                let s0: f64 = row.iter().zip(&col0).map(|(x, l)| x * l).sum();
                let s1: f64 = row.iter().zip(&col1).map(|(x, l)| x * l).sum();
                [s0, s1]
            })
            .collect();

        let mut data = Self::from_loadings_scores(&loadings_raw, &scores_raw)?;
        data.explained_ratio = explained_ratio;
        Ok(data)
    }

    pub fn from_loadings_scores(
        loadings: &[[f64; 2]],
        scores: &[[f64; 2]],
    ) -> Result<Self, MultivariateError> {
        if loadings.is_empty() {
            return Err(MultivariateError::Empty);
        }
        if loadings
            .iter()
            .chain(scores.iter())
            .flat_map(|p| p.iter())
            .any(|v| !v.is_finite())
        {
            return Err(MultivariateError::NoFiniteValues);
        }
        let loadings_raw = loadings.to_vec();
        let scores_raw = scores.to_vec();
        let (loadings_quiver, scores_data) = build_geometry(&loadings_raw, &scores_raw);
        let var_labels = (0..loadings_raw.len())
            .map(|i| format!("V{}", i + 1))
            .collect();
        let obs_labels = (0..scores_raw.len())
            .map(|i| format!("{}", i + 1))
            .collect();
        Ok(Self {
            loadings_raw,
            scores_raw,
            loadings_quiver,
            scores: scores_data,
            var_labels,
            obs_labels,
            explained_ratio: [0.0, 0.0],
        })
    }

    /// Kaiser 2-factor orthogonal Varimax rotation (preserves per-variable communalities).
    pub fn with_varimax(mut self) -> Self {
        let p = self.loadings_raw.len();
        if p < 2 {
            return self;
        }
        let mut h = Vec::with_capacity(p);
        for &[a, b] in &self.loadings_raw {
            h.push((a * a + b * b).sqrt().max(1e-12));
        }
        let mut sum_u = 0.0_f64;
        let mut sum_v = 0.0_f64;
        let mut sum_uv = 0.0_f64;
        let mut sum_u2_v2 = 0.0_f64;
        for (i, &[a, b]) in self.loadings_raw.iter().enumerate() {
            let x = a / h[i];
            let y = b / h[i];
            let u = x * x - y * y;
            let v = 2.0 * x * y;
            sum_u += u;
            sum_v += v;
            sum_uv += u * v;
            sum_u2_v2 += u * u - v * v;
        }
        let pf = p as f64;
        let num = 2.0 * (sum_uv - sum_u * sum_v / pf);
        let den = sum_u2_v2 - (sum_u * sum_u - sum_v * sum_v) / pf;
        let theta = 0.25 * num.atan2(den);
        let (s, c) = theta.sin_cos();

        for l in &mut self.loadings_raw {
            let x = l[0];
            let y = l[1];
            *l = [c * x + s * y, -s * x + c * y];
        }
        for sc in &mut self.scores_raw {
            let x = sc[0];
            let y = sc[1];
            *sc = [c * x + s * y, -s * x + c * y];
        }
        let (lq, sc_data) = build_geometry(&self.loadings_raw, &self.scores_raw);
        self.loadings_quiver = lq;
        self.scores = sc_data;
        self
    }

    pub fn with_var_labels(mut self, labels: impl Into<Vec<String>>) -> Self {
        let labels = labels.into();
        if labels.len() == self.loadings_raw.len() {
            self.var_labels = labels;
        }
        self
    }

    pub fn with_obs_labels(mut self, labels: impl Into<Vec<String>>) -> Self {
        let labels = labels.into();
        if labels.len() == self.scores_raw.len() {
            self.obs_labels = labels;
        }
        self
    }

    pub fn loadings_raw(&self) -> &[[f64; 2]] {
        &self.loadings_raw
    }
    pub fn scores_raw(&self) -> &[[f64; 2]] {
        &self.scores_raw
    }
    pub fn loadings_quiver(&self) -> &QuiverData {
        &self.loadings_quiver
    }
    pub fn scores(&self) -> &PlotData {
        &self.scores
    }
    pub fn var_labels(&self) -> &[String] {
        &self.var_labels
    }
    pub fn obs_labels(&self) -> &[String] {
        &self.obs_labels
    }
    pub fn explained_ratio(&self) -> [f64; 2] {
        self.explained_ratio
    }
    pub fn n_vars(&self) -> usize {
        self.loadings_raw.len()
    }
    pub fn n_obs(&self) -> usize {
        self.scores_raw.len()
    }
    pub fn sample_bytes(&self) -> usize {
        self.loadings_quiver.sample_bytes() + self.scores.sample_bytes()
    }

    pub fn bounds(&self, scales: [AxisScale; 2]) -> Option<PlotBounds> {
        let mut pts: Vec<PlotPoint> = self
            .loadings_raw
            .iter()
            .copied()
            .chain(std::iter::once([0.0, 0.0]))
            .chain(self.scores.points().iter().copied())
            .collect();
        if let Some((lo, hi)) = self.loadings_quiver.extent(scales) {
            pts.push(lo);
            pts.push(hi);
        }
        let mut min = [f64::INFINITY; 2];
        let mut max = [f64::NEG_INFINITY; 2];
        let mut count = 0usize;
        for pt in pts {
            if scales[0].forward(pt[0]).is_none() || scales[1].forward(pt[1]).is_none() {
                continue;
            }
            min[0] = min[0].min(pt[0]);
            min[1] = min[1].min(pt[1]);
            max[0] = max[0].max(pt[0]);
            max[1] = max[1].max(pt[1]);
            count += 1;
        }
        (count > 0)
            .then(|| PlotBounds::new(min, max).ok())
            .flatten()
    }
}

fn build_geometry(loadings: &[[f64; 2]], scores: &[[f64; 2]]) -> (QuiverData, PlotData) {
    let max_load = loadings
        .iter()
        .map(|l| l[0].abs().max(l[1].abs()))
        .fold(0.0_f64, f64::max)
        .max(1e-6);
    let max_score = scores
        .iter()
        .map(|s| s[0].abs().max(s[1].abs()))
        .fold(0.0_f64, f64::max);
    let score_scale = if max_score > 1e-12 {
        max_load / max_score
    } else {
        1.0
    };
    let arrows: Vec<QuiverArrow> = loadings
        .iter()
        .map(|&l| QuiverArrow::new([0.0, 0.0], l))
        .collect();
    let scaled_scores: Vec<PlotPoint> = scores
        .iter()
        .map(|&s| [s[0] * score_scale, s[1] * score_scale])
        .collect();
    (
        QuiverData::new(arrows),
        PlotData::new_unindexed(scaled_scores),
    )
}

fn apply_matlab_sign(col: &mut [f64]) {
    let mut best_abs = -1.0_f64;
    let mut sign = 1.0_f64;
    for &v in col.iter() {
        if v.abs() > best_abs {
            best_abs = v.abs();
            sign = if v < 0.0 { -1.0 } else { 1.0 };
        }
    }
    if sign < 0.0 {
        for v in col {
            *v = -*v;
        }
    }
}

fn jacobi_symmetric_eigen(n: usize, a: &mut [f64]) -> (Vec<f64>, Vec<f64>) {
    let mut v = vec![0.0_f64; n * n];
    for i in 0..n {
        v[i * n + i] = 1.0;
    }
    let max_iter = 50 * n * n;
    for _ in 0..max_iter {
        let mut p = 0usize;
        let mut q = 1usize;
        let mut max_off = 0.0_f64;
        for i in 0..n {
            for j in (i + 1)..n {
                let val = a[i * n + j].abs();
                if val > max_off {
                    max_off = val;
                    p = i;
                    q = j;
                }
            }
        }
        if max_off < 1e-13 {
            break;
        }
        let app = a[p * n + p];
        let aqq = a[q * n + q];
        let apq = a[p * n + q];
        let phi = 0.5 * (2.0 * apq).atan2(aqq - app);
        let (s, c) = phi.sin_cos();

        for i in 0..n {
            if i == p || i == q {
                continue;
            }
            let aip = a[i * n + p];
            let aiq = a[i * n + q];
            let nip = c * aip - s * aiq;
            let niq = s * aip + c * aiq;
            a[i * n + p] = nip;
            a[p * n + i] = nip;
            a[i * n + q] = niq;
            a[q * n + i] = niq;
        }
        a[p * n + p] = c * c * app - 2.0 * s * c * apq + s * s * aqq;
        a[q * n + q] = s * s * app + 2.0 * s * c * apq + c * c * aqq;
        a[p * n + q] = 0.0;
        a[q * n + p] = 0.0;

        for i in 0..n {
            let vip = v[i * n + p];
            let viq = v[i * n + q];
            v[i * n + p] = c * vip - s * viq;
            v[i * n + q] = s * vip + c * viq;
        }
    }
    let mut pairs: Vec<(usize, f64)> = (0..n).map(|i| (i, a[i * n + i])).collect();
    pairs.sort_by(|x, y| y.1.total_cmp(&x.1));
    let mut vals = Vec::with_capacity(n);
    let mut vecs = vec![0.0_f64; n * n];
    for (new_col, (old_col, val)) in pairs.into_iter().enumerate() {
        vals.push(val);
        for row in 0..n {
            vecs[row * n + new_col] = v[row * n + old_col];
        }
    }
    (vals, vecs)
}
