use std::{collections::HashMap, sync::Arc};

use super::{CachedExtents3D, TriMesh3DData, delaunay3d::delaunay_tetrahedra};
use crate::{AxisScale, Point3D, TriMeshError};

/// Host-owned 3D alpha-shape concave boundary mesh (`alphaShape(x,y,z,alpha)`, `EXT-10`).
#[derive(Clone, Debug)]
pub struct AlphaShape3DData {
    pub(crate) version: u64,
    mesh: Option<TriMesh3DData>,
    boundary_faces: Arc<[[u32; 3]]>,
    tetrahedra_count: usize,
    kept_tetrahedra: usize,
    alpha: f64,
    extents: CachedExtents3D,
}

impl AlphaShape3DData {
    pub fn from_sites(sites: &[Point3D], alpha: f64) -> Result<Self, TriMeshError> {
        if !alpha.is_finite() || alpha < 0.0 {
            return Err(TriMeshError::InvalidAlpha);
        }
        let (verts, tets, radii) = delaunay_tetrahedra(sites)?;
        let tetrahedra_count = tets.len();
        let mut face_map: HashMap<[u32; 3], (usize, [u32; 3], u32)> = HashMap::new();
        let mut kept_tetrahedra = 0usize;

        for (tet, &r) in tets.iter().zip(radii.iter()) {
            if r > alpha {
                continue;
            }
            kept_tetrahedra += 1;
            let [a, b, c, d] = *tet;
            for (mut f, opp) in [
                ([a, b, c], d),
                ([a, b, d], c),
                ([a, c, d], b),
                ([b, c, d], a),
            ] {
                let mut key = f;
                key.sort_unstable();
                orient_outward(&verts, &mut f, opp);
                face_map
                    .entry(key)
                    .and_modify(|e| e.0 += 1)
                    .or_insert((1, f, opp));
            }
        }

        let raw_boundary: Vec<[u32; 3]> = face_map
            .into_values()
            .filter_map(|(count, f, _)| (count == 1).then_some(f))
            .collect();

        let (mesh, compact_faces, extents) = if raw_boundary.is_empty() {
            (None, Vec::new(), CachedExtents3D::default())
        } else {
            let mut remap = vec![u32::MAX; verts.len()];
            let mut compact_verts = Vec::new();
            let mut compact_tris = Vec::with_capacity(raw_boundary.len());
            let mut ext = CachedExtents3D::default();
            for tri in raw_boundary {
                let mapped = tri.map(|orig| {
                    let slot = &mut remap[orig as usize];
                    if *slot == u32::MAX {
                        *slot = compact_verts.len() as u32;
                        let p = verts[orig as usize];
                        ext.push_point(p);
                        compact_verts.push(p);
                    }
                    *slot
                });
                compact_tris.push(mapped);
            }
            let trimesh = TriMesh3DData::new(compact_verts, compact_tris.clone())?;
            (Some(trimesh), compact_tris, ext)
        };

        Ok(Self {
            version: crate::data::points::next_version(),
            mesh,
            boundary_faces: Arc::from(compact_faces),
            tetrahedra_count,
            kept_tetrahedra,
            alpha,
            extents,
        })
    }

    pub fn critical_alpha(sites: &[Point3D]) -> Result<f64, TriMeshError> {
        let (verts, tets, radii) = delaunay_tetrahedra(sites)?;
        let n = verts.len();
        let mut order: Vec<usize> = (0..tets.len()).collect();
        order.sort_by(|&i, &j| radii[i].total_cmp(&radii[j]));
        let mut parent: Vec<usize> = (0..n).collect();
        let mut components = n;
        let mut seen = vec![false; n];
        let mut covered = 0usize;
        let mut crit = *radii.last().unwrap_or(&0.0);
        for idx in order {
            let t = tets[idx];
            for &v in &t {
                if !seen[v as usize] {
                    seen[v as usize] = true;
                    covered += 1;
                }
            }
            let r0 = uf_find(&mut parent, t[0] as usize);
            for &v in &t[1..] {
                let rv = uf_find(&mut parent, v as usize);
                if rv != r0 {
                    parent[rv] = r0;
                    components -= 1;
                }
            }
            crit = radii[idx];
            if covered == n && components == 1 {
                break;
            }
        }
        Ok(crit * (1.0 + 1e-6))
    }

    pub fn mesh(&self) -> Option<&TriMesh3DData> {
        self.mesh.as_ref()
    }
    pub fn boundary_faces(&self) -> &[[u32; 3]] {
        &self.boundary_faces
    }
    pub fn tetrahedra_count(&self) -> usize {
        self.tetrahedra_count
    }
    pub fn kept_tetrahedra(&self) -> usize {
        self.kept_tetrahedra
    }
    pub fn alpha(&self) -> f64 {
        self.alpha
    }
    pub fn sample_bytes(&self) -> usize {
        self.mesh.as_ref().map_or(0, TriMesh3DData::sample_bytes)
            + std::mem::size_of_val(self.boundary_faces.as_ref())
    }
    pub(crate) fn allocation(&self) -> (usize, usize) {
        (self.boundary_faces.as_ptr() as usize, self.sample_bytes())
    }
    pub(crate) fn extent(&self, scales: [AxisScale; 3]) -> Option<[Point3D; 2]> {
        self.extents.extent(scales)
    }
    pub(crate) fn domain_invalid(&self, scales: [AxisScale; 3]) -> usize {
        self.extents.domain_invalid(scales)
    }
}

fn orient_outward(verts: &[Point3D], face: &mut [u32; 3], opp: u32) {
    let pa = verts[face[0] as usize];
    let pb = verts[face[1] as usize];
    let pc = verts[face[2] as usize];
    let pd = verts[opp as usize];
    let ab = [pb[0] - pa[0], pb[1] - pa[1], pb[2] - pa[2]];
    let ac = [pc[0] - pa[0], pc[1] - pa[1], pc[2] - pa[2]];
    let ad = [pd[0] - pa[0], pd[1] - pa[1], pd[2] - pa[2]];
    let n = [
        ab[1] * ac[2] - ab[2] * ac[1],
        ab[2] * ac[0] - ab[0] * ac[2],
        ab[0] * ac[1] - ab[1] * ac[0],
    ];
    if n[0] * ad[0] + n[1] * ad[1] + n[2] * ad[2] > 0.0 {
        face.swap(1, 2);
    }
}

fn uf_find(parent: &mut [usize], mut x: usize) -> usize {
    while parent[x] != x {
        parent[x] = parent[parent[x]];
        x = parent[x];
    }
    x
}
