use std::sync::Arc;

use egui::Pos2;

use super::{CachedExtents3D, Plot3DError};
use crate::{AxisScale, Point3D, Transform3D};

#[derive(Clone, Debug)]
struct OctreeNode {
    min: Point3D,
    max: Point3D,
    rep_index: u32,
    start: u32,
    len: u32,
    children: [u32; 8],
}

/// High-density 3D XYZ/RGB/Intensity point cloud with built-in octree LOD & picking (`pcshow`, `EXT-24`).
#[derive(Clone, Debug)]
pub struct PointCloud3DData {
    points: Arc<[Point3D]>,
    rgb: Option<Arc<[[u8; 3]]>>,
    intensity: Option<Arc<[f32]>>,
    intensity_range: Option<[f32; 2]>,
    summary: CachedExtents3D,
    order: Vec<u32>,
    nodes: Vec<OctreeNode>,
    non_finite: usize,
}

impl PointCloud3DData {
    pub fn from_xyz(points: impl Into<Arc<[Point3D]>>) -> Result<Self, Plot3DError> {
        let points = points.into();
        if points.is_empty() {
            return Err(Plot3DError::EmptyInput);
        }
        let mut summary = CachedExtents3D::default();
        let mut order = Vec::with_capacity(points.len());
        let mut lo = [f64::INFINITY; 3];
        let mut hi = [f64::NEG_INFINITY; 3];
        let mut non_finite = 0usize;
        for (i, &p) in points.iter().enumerate() {
            summary.push_point(p);
            if p.iter().all(|v| v.is_finite()) {
                order.push(i as u32);
                for a in 0..3 {
                    lo[a] = lo[a].min(p[a]);
                    hi[a] = hi[a].max(p[a]);
                }
            } else {
                non_finite += 1;
            }
        }
        if order.is_empty() {
            return Err(Plot3DError::EmptyInput);
        }
        for a in 0..3 {
            if (hi[a] - lo[a]).abs() < 1e-12 {
                hi[a] = lo[a] + 1.0;
            }
        }
        let mut nodes = Vec::new();
        let len = order.len() as u32;
        build_octree(
            &points,
            &mut order,
            OctreeCell {
                start: 0,
                len,
                min: lo,
                max: hi,
                depth: 0,
            },
            &mut nodes,
        );
        Ok(Self {
            points,
            rgb: None,
            intensity: None,
            intensity_range: None,
            summary,
            order,
            nodes,
            non_finite,
        })
    }

    pub fn with_rgb(mut self, rgb: impl Into<Arc<[[u8; 3]]>>) -> Result<Self, Plot3DError> {
        let rgb = rgb.into();
        if rgb.len() != self.points.len() {
            return Err(Plot3DError::LengthMismatch);
        }
        self.rgb = Some(rgb);
        Ok(self)
    }

    pub fn with_intensity(mut self, vals: impl Into<Arc<[f32]>>) -> Result<Self, Plot3DError> {
        let vals = vals.into();
        if vals.len() != self.points.len() {
            return Err(Plot3DError::LengthMismatch);
        }
        let (mut lo, mut hi) = (f32::INFINITY, f32::NEG_INFINITY);
        for &v in vals.iter() {
            if v.is_finite() {
                lo = lo.min(v);
                hi = hi.max(v);
            }
        }
        self.intensity_range = (lo <= hi).then_some(if (hi - lo).abs() > 1e-9 {
            [lo, hi]
        } else {
            [lo, lo + 1.0]
        });
        self.intensity = Some(vals);
        Ok(self)
    }

    pub fn points(&self) -> &[Point3D] {
        &self.points
    }
    pub fn rgb(&self) -> Option<&[[u8; 3]]> {
        self.rgb.as_deref()
    }
    pub fn intensity(&self) -> Option<&[f32]> {
        self.intensity.as_deref()
    }
    pub fn intensity_range(&self) -> Option<[f32; 2]> {
        self.intensity_range
    }
    pub fn octree_node_count(&self) -> usize {
        self.nodes.len()
    }
    pub fn non_finite(&self) -> usize {
        self.non_finite
    }
    pub fn len(&self) -> usize {
        self.points.len()
    }
    pub fn is_empty(&self) -> bool {
        self.points.is_empty()
    }
    pub(crate) fn allocation(&self) -> (usize, usize) {
        (
            self.points.as_ptr() as usize,
            std::mem::size_of_val(&*self.points)
                + self.rgb.as_ref().map_or(0, |r| std::mem::size_of_val(&**r))
                + self
                    .intensity
                    .as_ref()
                    .map_or(0, |v| std::mem::size_of_val(&**v)),
        )
    }
    pub(crate) fn extent(&self, scales: [AxisScale; 3]) -> Option<[Point3D; 2]> {
        self.summary.extent(scales)
    }
    pub(crate) fn domain_invalid(&self, scales: [AxisScale; 3]) -> usize {
        self.summary.domain_invalid(scales)
    }

    /// Frustum-culled screen-size-adaptive octree LOD returning at most `max_points` indices.
    pub fn query_lod(
        &self,
        transform: Transform3D,
        max_points: usize,
        pixel_cell: f32,
    ) -> Vec<usize> {
        if self.nodes.is_empty() || max_points == 0 {
            return Vec::new();
        }
        let mut out = Vec::with_capacity(max_points.min(self.order.len()));
        let mut stack = vec![0u32];
        while let Some(ni) = stack.pop() {
            if out.len() >= max_points {
                break;
            }
            let node = &self.nodes[ni as usize];
            let Some(diag) = projected_box_diag(node.min, node.max, transform) else {
                continue;
            };
            if node.children == [u32::MAX; 8] {
                let stride = (node.len as usize * 2 / (max_points - out.len()).max(1)).max(1);
                for k in (0..node.len as usize).step_by(stride) {
                    if out.len() >= max_points {
                        break;
                    }
                    out.push(self.order[node.start as usize + k] as usize);
                }
            } else if diag < pixel_cell.max(1.5) {
                out.push(node.rep_index as usize);
            } else {
                for &child in &node.children {
                    if child != u32::MAX {
                        stack.push(child);
                    }
                }
            }
        }
        out
    }

    /// Octree branch-and-bound nearest point picking in screen pixels.
    pub fn pick_nearest(
        &self,
        transform: Transform3D,
        pointer: Pos2,
        radius: f32,
    ) -> Option<(usize, Point3D, f32, f32)> {
        if self.nodes.is_empty() {
            return None;
        }
        let mut best: Option<(usize, Point3D, f32, f32)> = None;
        let mut stack = vec![0u32];
        while let Some(ni) = stack.pop() {
            let node = &self.nodes[ni as usize];
            if !box_near_pointer(node.min, node.max, transform, pointer, radius) {
                continue;
            }
            if node.children == [u32::MAX; 8] {
                for k in 0..node.len as usize {
                    let idx = self.order[node.start as usize + k] as usize;
                    let p = self.points[idx];
                    if let Some((pos, depth)) = transform.project_data(p) {
                        let d = pos.distance(pointer);
                        if d <= radius && best.as_ref().is_none_or(|b| d < b.2) {
                            best = Some((idx, p, d, depth));
                        }
                    }
                }
            } else {
                for &c in &node.children {
                    if c != u32::MAX {
                        stack.push(c);
                    }
                }
            }
        }
        best
    }
}

struct OctreeCell {
    start: u32,
    len: u32,
    min: Point3D,
    max: Point3D,
    depth: u8,
}

fn build_octree(
    points: &[Point3D],
    order: &mut [u32],
    cell: OctreeCell,
    nodes: &mut Vec<OctreeNode>,
) -> u32 {
    let OctreeCell {
        start,
        len,
        min,
        max,
        depth,
    } = cell;
    let idx = nodes.len() as u32;
    let rep_index = order[start as usize];
    nodes.push(OctreeNode {
        min,
        max,
        rep_index,
        start,
        len,
        children: [u32::MAX; 8],
    });
    if len <= 64 || depth >= 7 {
        return idx;
    }
    let mid = [
        0.5 * (min[0] + max[0]),
        0.5 * (min[1] + max[1]),
        0.5 * (min[2] + max[2]),
    ];
    let slice = &mut order[start as usize..(start + len) as usize];
    slice.sort_by_key(|&pi| {
        let p = points[pi as usize];
        usize::from(p[0] >= mid[0])
            | (usize::from(p[1] >= mid[1]) << 1)
            | (usize::from(p[2] >= mid[2]) << 2)
    });

    let mut children = [u32::MAX; 8];
    let mut cursor = start;
    for (oct, slot) in children.iter_mut().enumerate() {
        let c_start = cursor;
        while (cursor - start) < len {
            let p = points[order[cursor as usize] as usize];
            let code = usize::from(p[0] >= mid[0])
                | (usize::from(p[1] >= mid[1]) << 1)
                | (usize::from(p[2] >= mid[2]) << 2);
            if code != oct {
                break;
            }
            cursor += 1;
        }
        let c_len = cursor - c_start;
        if c_len > 0 {
            let c_min = [
                if oct & 1 == 0 { min[0] } else { mid[0] },
                if oct & 2 == 0 { min[1] } else { mid[1] },
                if oct & 4 == 0 { min[2] } else { mid[2] },
            ];
            let c_max = [
                if oct & 1 == 0 { mid[0] } else { max[0] },
                if oct & 2 == 0 { mid[1] } else { max[1] },
                if oct & 4 == 0 { mid[2] } else { max[2] },
            ];
            *slot = build_octree(
                points,
                order,
                OctreeCell {
                    start: c_start,
                    len: c_len,
                    min: c_min,
                    max: c_max,
                    depth: depth + 1,
                },
                nodes,
            );
        }
    }
    nodes[idx as usize].children = children;
    idx
}

fn projected_box_diag(min: Point3D, max: Point3D, transform: Transform3D) -> Option<f32> {
    let mut smin = Pos2::new(f32::INFINITY, f32::INFINITY);
    let mut smax = Pos2::new(f32::NEG_INFINITY, f32::NEG_INFINITY);
    let mut any_in = false;
    let mut corners = 0u8;
    for mask in 0..8 {
        let p = [
            if mask & 1 == 0 { min[0] } else { max[0] },
            if mask & 2 == 0 { min[1] } else { max[1] },
            if mask & 4 == 0 { min[2] } else { max[2] },
        ];
        if let Some(u) = transform.data_to_unit(p)
            && let Some((pos, _)) = transform.project_unit(u)
        {
            if (-0.02..=1.02).contains(&u[0])
                && (-0.02..=1.02).contains(&u[1])
                && (-0.02..=1.02).contains(&u[2])
            {
                any_in = true;
            }
            smin = smin.min(pos);
            smax = smax.max(pos);
            corners += 1;
        }
    }
    if corners == 8
        && !egui::Rect::from_min_max(smin, smax).intersects(transform.rect().expand(8.0))
    {
        return None;
    }
    (any_in && smin.x.is_finite()).then(|| smin.distance(smax))
}

fn box_near_pointer(
    min: Point3D,
    max: Point3D,
    transform: Transform3D,
    pointer: Pos2,
    radius: f32,
) -> bool {
    let mut smin = Pos2::new(f32::INFINITY, f32::INFINITY);
    let mut smax = Pos2::new(f32::NEG_INFINITY, f32::NEG_INFINITY);
    let mut any = false;
    for mask in 0..8 {
        let p = [
            if mask & 1 == 0 { min[0] } else { max[0] },
            if mask & 2 == 0 { min[1] } else { max[1] },
            if mask & 4 == 0 { min[2] } else { max[2] },
        ];
        if let Some((pos, _)) = transform.project_data(p) {
            smin = smin.min(pos);
            smax = smax.max(pos);
            any = true;
        }
    }
    any && egui::Rect::from_min_max(smin, smax)
        .expand(radius)
        .contains(pointer)
}
