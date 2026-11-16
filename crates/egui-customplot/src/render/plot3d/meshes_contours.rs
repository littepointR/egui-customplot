use egui::{Color32, Stroke};

use super::{
    scene::Scene3D,
    surfaces::{is_fin3, map_scalar_color},
};
use crate::{
    ColorScale, Contour3D, Contour3DData, MeshSurface3D, MeshSurfaceStyle, Plot3DDiagnostics,
    Transform3D,
};

pub(super) fn collect_mesh_surface3d(
    scene: &mut Scene3D,
    m: MeshSurface3D<'_>,
    bg_color: Color32,
    fallback: Color32,
    transform: Transform3D,
    diagnostics: &mut Plot3DDiagnostics,
) {
    let data = m.data;
    let rows = data.rows();
    let cols = data.cols();
    let pts = data.points();
    let scals = data.scalars();
    let s_range = data.scalar_range();
    let width = m.style.line_width.unwrap_or(1.1);
    let occlude = Color32::from_rgb(bg_color.r(), bg_color.g(), bg_color.b());
    let z_ref = m.base_z.unwrap_or_else(|| transform.bounds().min()[2]);

    let seg_color = |i0: usize, i1: usize| -> Color32 {
        m.style.color.unwrap_or_else(|| {
            map_scalar_color(
                0.5 * (scals[i0] + scals[i1]),
                m.color_scale,
                s_range,
                fallback,
            )
        })
    };

    match m.mesh_style {
        MeshSurfaceStyle::Grid | MeshSurfaceStyle::CurtainZ => {
            for r in 0..(rows - 1) {
                for c in 0..(cols - 1) {
                    diagnostics.visited_items += 1;
                    let i00 = r * cols + c;
                    let i10 = i00 + 1;
                    let i01 = (r + 1) * cols + c;
                    let i11 = i01 + 1;
                    let p00 = pts[i00];
                    let p10 = pts[i10];
                    let p01 = pts[i01];
                    let p11 = pts[i11];
                    if ![p00, p10, p01, p11].iter().all(|p| is_fin3(*p)) {
                        continue;
                    }
                    if m.hidden_line {
                        scene.push_gouraud_tri_data(
                            [p00, p10, p11],
                            [occlude; 3],
                            1.5e-4,
                            transform,
                        );
                        scene.push_gouraud_tri_data(
                            [p00, p11, p01],
                            [occlude; 3],
                            1.5e-4,
                            transform,
                        );
                    }
                    scene.push_data_segment(
                        p00,
                        p10,
                        Stroke::new(width, seg_color(i00, i10)),
                        transform,
                    );
                    scene.push_data_segment(
                        p00,
                        p01,
                        Stroke::new(width, seg_color(i00, i01)),
                        transform,
                    );
                    if c + 2 == cols {
                        scene.push_data_segment(
                            p10,
                            p11,
                            Stroke::new(width, seg_color(i10, i11)),
                            transform,
                        );
                    }
                    if r + 2 == rows {
                        scene.push_data_segment(
                            p01,
                            p11,
                            Stroke::new(width, seg_color(i01, i11)),
                            transform,
                        );
                    }
                }
            }
            if m.mesh_style == MeshSurfaceStyle::CurtainZ {
                let mut curtain_seg = |ia: usize, ib: usize| {
                    let pa = pts[ia];
                    let pb = pts[ib];
                    if !is_fin3(pa) || !is_fin3(pb) {
                        return;
                    }
                    let ba = [pa[0], pa[1], z_ref];
                    let bb = [pb[0], pb[1], z_ref];
                    if m.hidden_line {
                        scene.push_gouraud_tri_data([pa, pb, bb], [occlude; 3], 1.5e-4, transform);
                        scene.push_gouraud_tri_data([pa, bb, ba], [occlude; 3], 1.5e-4, transform);
                    }
                    let stroke = Stroke::new(width, seg_color(ia, ib));
                    scene.push_data_segment(pa, ba, stroke, transform);
                    scene.push_data_segment(ba, bb, stroke, transform);
                };
                for c in 0..(cols - 1) {
                    curtain_seg(c, c + 1);
                    curtain_seg((rows - 1) * cols + c, (rows - 1) * cols + c + 1);
                }
                for r in 0..(rows - 1) {
                    curtain_seg(r * cols, (r + 1) * cols);
                    curtain_seg(r * cols + cols - 1, (r + 1) * cols + cols - 1);
                }
            }
        }
        MeshSurfaceStyle::Waterfall => {
            for r in 0..rows {
                diagnostics.visited_items += 1;
                for c in 0..(cols - 1) {
                    let i0 = r * cols + c;
                    let i1 = i0 + 1;
                    let pa = pts[i0];
                    let pb = pts[i1];
                    if !is_fin3(pa) || !is_fin3(pb) {
                        continue;
                    }
                    if m.hidden_line {
                        let ba = [pa[0], pa[1], z_ref];
                        let bb = [pb[0], pb[1], z_ref];
                        scene.push_gouraud_tri_data([pa, pb, bb], [occlude; 3], 1.5e-4, transform);
                        scene.push_gouraud_tri_data([pa, bb, ba], [occlude; 3], 1.5e-4, transform);
                    }
                    scene.push_data_segment(
                        pa,
                        pb,
                        Stroke::new(width, seg_color(i0, i1)),
                        transform,
                    );
                }
                let p_first = pts[r * cols];
                if is_fin3(p_first) {
                    let s = Stroke::new(width, seg_color(r * cols, r * cols));
                    scene.push_data_segment(p_first, [p_first[0], p_first[1], z_ref], s, transform);
                }
                let p_last = pts[r * cols + cols - 1];
                if is_fin3(p_last) {
                    let idx = r * cols + cols - 1;
                    let s = Stroke::new(width, seg_color(idx, idx));
                    scene.push_data_segment(p_last, [p_last[0], p_last[1], z_ref], s, transform);
                }
            }
        }
    }

    if m.floor_contours
        && let Some(c) = data.contours()
    {
        let z_floor = m.floor_z.unwrap_or(z_ref);
        collect_floor_contours(scene, c, z_floor, m.color_scale, fallback, transform);
    }
}

pub(super) fn collect_contour3d(
    scene: &mut Scene3D,
    c3: Contour3D<'_>,
    fallback: Color32,
    transform: Transform3D,
    diagnostics: &mut Plot3DDiagnostics,
) {
    let lvls = c3.data.levels();
    let range = (lvls.len() >= 2).then(|| [lvls[0], lvls[lvls.len() - 1]]);
    let w = c3.style.line_width.unwrap_or(1.4);
    for line in c3.data.lines() {
        diagnostics.visited_items += 1;
        let col = c3
            .style
            .color
            .unwrap_or_else(|| map_scalar_color(line.level, c3.color_scale, range, fallback));
        let stroke = Stroke::new(w, col);
        for seg in line.points.windows(2) {
            scene.push_data_segment(seg[0], seg[1], stroke, transform);
        }
    }
}

pub(super) fn collect_floor_contours(
    scene: &mut Scene3D,
    c: &Contour3DData,
    z_floor: f64,
    cs: Option<ColorScale>,
    fallback: Color32,
    transform: Transform3D,
) {
    let lvls = c.levels();
    let range = (lvls.len() >= 2).then(|| [lvls[0], lvls[lvls.len() - 1]]);
    for line in c.lines() {
        let col = map_scalar_color(line.level, cs, range, fallback);
        let stroke = Stroke::new(1.1, col);
        for seg in line.points.windows(2) {
            scene.push_data_segment(
                [seg[0][0], seg[0][1], z_floor],
                [seg[1][0], seg[1][1], z_floor],
                stroke,
                transform,
            );
        }
    }
}
