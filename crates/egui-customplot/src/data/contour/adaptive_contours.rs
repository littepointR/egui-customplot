use super::{
    ContourData, ContourError, ContourLevels, ContourLine, FilledContourData,
    adaptive::sample_adaptive_triangles,
    extract::{
        BandAccumulator, Node, chain_segments, resolve_band_levels_from_range,
        resolve_line_levels_from_range, tri_isoline,
    },
};

impl ContourData {
    pub fn from_function_adaptive(
        x_range: [f64; 2],
        y_range: [f64; 2],
        levels: ContourLevels,
        max_evaluations: usize,
        f: impl FnMut(f64, f64) -> f64,
    ) -> Result<Self, ContourError> {
        let (verts, tris, range) = sample_adaptive_triangles(x_range, y_range, max_evaluations, f)?;
        let levels = resolve_line_levels_from_range(range, levels)?;
        if levels.is_empty() {
            return Ok(Self::from_raw_lines(levels, Vec::new()));
        }
        let mut per_level = vec![Vec::new(); levels.len()];
        for [i0, i1, i2] in tris {
            let tri = [i0, i1, i2].map(|i| {
                let p = verts[i as usize];
                Node {
                    p: [p[0], p[1]],
                    z: p[2],
                }
            });
            let z_min = tri[0].z.min(tri[1].z).min(tri[2].z);
            let z_max = tri[0].z.max(tri[1].z).max(tri[2].z);
            let start = levels.partition_point(|&l| l < z_min);
            let end = levels.partition_point(|&l| l <= z_max);
            for li in start..end {
                if let Some(seg) = tri_isoline(tri, levels[li]) {
                    per_level[li].push(seg);
                }
            }
        }
        let mut lines = Vec::new();
        for (level, segs) in levels.iter().copied().zip(per_level) {
            for points in chain_segments(segs) {
                lines.push(ContourLine {
                    level,
                    points: points.into_boxed_slice(),
                });
            }
        }
        Ok(Self::from_raw_lines(levels, lines))
    }
}

impl FilledContourData {
    pub fn from_function_adaptive(
        x_range: [f64; 2],
        y_range: [f64; 2],
        levels: ContourLevels,
        max_evaluations: usize,
        f: impl FnMut(f64, f64) -> f64,
    ) -> Result<Self, ContourError> {
        let (verts, tris, range) = sample_adaptive_triangles(x_range, y_range, max_evaluations, f)?;
        let (levels, band_bounds) = resolve_band_levels_from_range(range, levels)?;
        let mut acc = BandAccumulator::new(&band_bounds);
        for [i0, i1, i2] in tris {
            let tri = [i0, i1, i2].map(|i| {
                let p = verts[i as usize];
                Node {
                    p: [p[0], p[1]],
                    z: p[2],
                }
            });
            acc.push_triangle(tri);
        }
        Ok(Self::from_raw_bands(levels, acc.finish()))
    }
}
