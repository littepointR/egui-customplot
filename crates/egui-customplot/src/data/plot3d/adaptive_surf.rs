use super::TriMesh3DData;
use crate::{
    FunctionContourError, TriMeshError, data::contour::adaptive::sample_adaptive_triangles,
};

impl TriMesh3DData {
    /// Adaptive gradient-driven `fsurf` triangle mesh (`EXT-12`) sharing the crack-free quadtree.
    pub fn from_explicit_fn_adaptive(
        x_range: [f64; 2],
        y_range: [f64; 2],
        max_evaluations: usize,
        f: impl FnMut(f64, f64) -> f64,
    ) -> Result<Self, FunctionContourError> {
        let (raw_verts, raw_tris, _) =
            sample_adaptive_triangles(x_range, y_range, max_evaluations, f)?;
        let mut remap = vec![u32::MAX; raw_verts.len()];
        let mut compact_verts = Vec::with_capacity(raw_verts.len());
        let mut compact_tris = Vec::with_capacity(raw_tris.len());
        for t in raw_tris {
            let out = t.map(|orig| {
                let slot = &mut remap[orig as usize];
                if *slot == u32::MAX {
                    *slot = compact_verts.len() as u32;
                    compact_verts.push(raw_verts[orig as usize]);
                }
                *slot
            });
            compact_tris.push(out);
        }
        TriMesh3DData::new(compact_verts, compact_tris).map_err(|err| match err {
            TriMeshError::InsufficientPoints | TriMeshError::InvalidTriangleIndex => {
                FunctionContourError::InvalidResolution
            }
            _ => FunctionContourError::InvalidResolution,
        })
    }
}
