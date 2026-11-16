use egui::{Color32, Stroke};

use super::{
    layers::{collect_polyline, push_marker_data},
    scene::{Scene3D, scale_rgb},
    surfaces::{
        SurfaceAppearance, collect_trisurf3d, emit_surface_grid, light_factor, map_scalar_color,
    },
};
use crate::{
    ConePlot3D, Isocaps3D, Isosurface3D, LightSource3D, MarkerShape, Plot3DDiagnostics,
    SeriesStyle, StreamParticles3D, StreamRibbon3D, StreamTube3D, Streamline3D, SurfaceShading,
    TetraMesh3D, Transform3D, TriMesh3DData, TriSurf3D, VolumeSlice3D, model::ResolvedPlotStyle,
};

pub(super) fn collect_volume_slice3d(
    scene: &mut Scene3D,
    vs: VolumeSlice3D<'_>,
    fallback: Color32,
    transform: Transform3D,
    diagnostics: &mut Plot3DDiagnostics,
) {
    let app = SurfaceAppearance {
        shading: vs.shading,
        explicit_color: vs.style.color,
        color_scale: vs.color_scale,
        light: None,
        colormap_lighting: false,
        wireframe_stroke: vs.wireframe_stroke,
        alpha: vs.alpha,
        fallback,
    };
    for slice in vs.data.slices() {
        emit_surface_grid(scene, slice, &app, transform, diagnostics);
    }
}

#[allow(clippy::too_many_arguments)]
fn collect_mesh_overlay(
    scene: &mut Scene3D,
    mesh: Option<&TriMesh3DData>,
    shading: SurfaceShading,
    color_scale: Option<crate::ColorScale>,
    alpha: f32,
    style: SeriesStyle,
    bg_color: Color32,
    fallback: Color32,
    transform: Transform3D,
    diagnostics: &mut Plot3DDiagnostics,
) {
    if mesh.is_none() {
        return;
    }
    let ts = TriSurf3D {
        data: mesh,
        implicit: None,
        name: "",
        id_salt: None,
        shading,
        wireframe: false,
        hidden_line: false,
        color_scale,
        edge_stroke: None,
        alpha,
        style,
    };
    collect_trisurf3d(scene, ts, bg_color, fallback, transform, diagnostics);
}

pub(super) fn collect_isosurface3d(
    scene: &mut Scene3D,
    iso: Isosurface3D<'_>,
    bg_color: Color32,
    fallback: Color32,
    transform: Transform3D,
    diagnostics: &mut Plot3DDiagnostics,
) {
    collect_mesh_overlay(
        scene,
        iso.data.mesh(),
        iso.shading,
        iso.color_scale,
        iso.alpha,
        iso.style,
        bg_color,
        fallback,
        transform,
        diagnostics,
    );
}

pub(super) fn collect_isocaps3d(
    scene: &mut Scene3D,
    caps: Isocaps3D<'_>,
    bg_color: Color32,
    fallback: Color32,
    transform: Transform3D,
    diagnostics: &mut Plot3DDiagnostics,
) {
    collect_mesh_overlay(
        scene,
        caps.data.mesh(),
        caps.shading,
        caps.color_scale,
        caps.alpha,
        caps.style,
        bg_color,
        fallback,
        transform,
        diagnostics,
    );
}

pub(super) fn collect_cone_plot3d(
    scene: &mut Scene3D,
    cp: ConePlot3D<'_>,
    bg_color: Color32,
    fallback: Color32,
    transform: Transform3D,
    diagnostics: &mut Plot3DDiagnostics,
) {
    collect_mesh_overlay(
        scene,
        cp.data.mesh(),
        cp.shading,
        cp.color_scale,
        cp.alpha,
        cp.style,
        bg_color,
        fallback,
        transform,
        diagnostics,
    );
}

pub(super) fn collect_tetramesh3d(
    scene: &mut Scene3D,
    tm: TetraMesh3D<'_>,
    fallback: Color32,
    transform: Transform3D,
    diagnostics: &mut Plot3DDiagnostics,
) {
    let data = tm.data;
    let verts = data.vertices();
    let (faces, tids) = if tm.boundary_only {
        (data.boundary_faces(), data.boundary_face_tet())
    } else {
        (data.all_faces(), data.all_face_tet())
    };
    let scals = data.cell_scalars();
    let s_range = data.scalar_range();
    let light = LightSource3D::default();

    for (&[a, b, c], &tid) in faces.iter().zip(tids.iter()) {
        diagnostics.visited_items += 1;
        let (pa, pb, pc) = (verts[a as usize], verts[b as usize], verts[c as usize]);
        let ab = [pb[0] - pa[0], pb[1] - pa[1], pb[2] - pa[2]];
        let ac = [pc[0] - pa[0], pc[1] - pa[1], pc[2] - pa[2]];
        let n = [
            ab[1] * ac[2] - ab[2] * ac[1],
            ab[2] * ac[0] - ab[0] * ac[2],
            ab[0] * ac[1] - ab[1] * ac[0],
        ];
        let base = tm.style.color.unwrap_or_else(|| {
            map_scalar_color(scals[tid as usize], tm.color_scale, s_range, fallback)
        });
        let col = scale_rgb(base, light_factor(n, light, transform)).gamma_multiply(tm.alpha);
        scene.push_gouraud_tri_data([pa, pb, pc], [col; 3], 0.0, transform);
    }

    if let Some(stroke) = tm.edge_stroke {
        for &[a, b] in data.edges() {
            scene.push_data_segment(verts[a as usize], verts[b as usize], stroke, transform);
        }
    }
}

pub(super) fn collect_streamline3d(
    scene: &mut Scene3D,
    sl: Streamline3D<'_>,
    fallback: Color32,
    transform: Transform3D,
    diagnostics: &mut Plot3DDiagnostics,
) {
    let w = sl.style.line_width.unwrap_or(1.5);
    let s_range = sl.data.speed_range();
    for line in sl.data.lines() {
        let n = line.points.len();
        if n < 2 {
            continue;
        }
        for i in 0..(n - 1) {
            diagnostics.visited_items += 1;
            let spd = 0.5 * (line.speeds[i] + line.speeds[i + 1]);
            let col = sl
                .style
                .color
                .unwrap_or_else(|| map_scalar_color(spd, sl.color_scale, s_range, fallback));
            scene.push_data_segment(
                line.points[i],
                line.points[i + 1],
                Stroke::new(w, col),
                transform,
            );
        }
    }
}

pub(super) fn collect_streamparticles3d(
    scene: &mut Scene3D,
    sp: StreamParticles3D<'_>,
    fallback: Color32,
    transform: Transform3D,
    diagnostics: &mut Plot3DDiagnostics,
) {
    let line_col = sp.style.color.unwrap_or(fallback).gamma_multiply(0.35);
    if sp.show_lines {
        for line in sp.data.streamlines().lines() {
            collect_polyline(
                scene,
                &line.points,
                Stroke::new(1.0, line_col),
                None,
                transform,
                diagnostics,
            );
        }
    }
    let rad = sp.style.marker_radius.unwrap_or(4.0);
    let shape = sp.style.marker_shape.unwrap_or(MarkerShape::Circle);
    let s_range = sp.data.streamlines().speed_range();
    for (pos, spd) in sp.playback.positions(sp.data) {
        diagnostics.visited_items += 1;
        let col = sp
            .style
            .color
            .unwrap_or_else(|| map_scalar_color(spd, sp.color_scale, s_range, fallback));
        push_marker_data(scene, pos, rad, col, shape, transform);
    }
}

pub(super) fn collect_stream_ribbon3d(
    scene: &mut Scene3D,
    sr: StreamRibbon3D<'_>,
    style: &ResolvedPlotStyle<'_>,
    transform: Transform3D,
    diagnostics: &mut Plot3DDiagnostics,
) {
    for (idx, strip) in sr.data.strips().iter().enumerate() {
        let def_c = style.resolve_series(idx, Default::default()).stroke.color;
        let app = SurfaceAppearance {
            shading: sr.shading,
            explicit_color: sr.style.color,
            color_scale: sr.color_scale,
            light: Some(LightSource3D::default()),
            colormap_lighting: false,
            wireframe_stroke: sr.edge_stroke,
            alpha: sr.alpha,
            fallback: def_c,
        };
        emit_surface_grid(scene, strip, &app, transform, diagnostics);
    }
}

pub(super) fn collect_stream_tube3d(
    scene: &mut Scene3D,
    st: StreamTube3D<'_>,
    style: &ResolvedPlotStyle<'_>,
    transform: Transform3D,
    diagnostics: &mut Plot3DDiagnostics,
) {
    for (idx, tube) in st.data.tubes().iter().enumerate() {
        let def_c = style.resolve_series(idx, Default::default()).stroke.color;
        let app = SurfaceAppearance {
            shading: st.shading,
            explicit_color: st.style.color,
            color_scale: st.color_scale,
            light: Some(LightSource3D::default()),
            colormap_lighting: false,
            wireframe_stroke: st.edge_stroke,
            alpha: st.alpha,
            fallback: def_c,
        };
        emit_surface_grid(scene, tube, &app, transform, diagnostics);
    }
}
