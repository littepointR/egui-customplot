#[path = "support/input.rs"]
mod support;

use std::sync::Arc;

use egui::{Color32, Context, Pos2, Rect, Shape, vec2};
use egui_customplot::{
    AxisScale, Bounds3D, Camera3D, ColorMap, ColorScale, ConstantPlane3D, DataAspect3D, DepthMode,
    Layer3D, Plot3D, Plot3DState, PointCloud3DData, Surface3D, Surface3DData, Text3D, Transform3D,
    TriMesh3DData, TriSurf3D,
};

#[test]
fn enclosing_octree_nodes_keep_visible_points_and_exclude_clipped_picks() {
    let cloud = PointCloud3DData::from_xyz(vec![[0.0, 0.0, 10.0], [-10.0; 3], [10.0; 3], [0.0; 3]])
        .unwrap();
    let transform = Transform3D::new(
        Bounds3D::new([-1.0; 3], [1.0; 3]).unwrap(),
        [AxisScale::Linear; 3],
        [false; 3],
        DataAspect3D::Auto,
        Rect::from_min_size(Pos2::ZERO, vec2(300.0, 300.0)),
        Camera3D::view_2d_xy(),
    )
    .unwrap();
    let center = transform.project_data([0.0; 3]).unwrap().0;
    assert_eq!(cloud.query_lod(transform, 1, 2.0), [3]);
    assert_eq!(cloud.pick_nearest(transform, center, 1.0).unwrap().0, 3);
}

#[test]
fn changing_surface_colors_invalidates_cached_pixels() {
    let ctx = Context::default();
    let surface = Surface3DData::from_grid(
        &[0.0, 1.0, 2.0],
        &[0.0, 1.0, 2.0],
        &[0.0, 0.5, 0.7, 0.3, 0.4, 0.8, 0.1, 0.5, 0.9],
    )
    .unwrap();
    let overlays = Overlays::default();
    let colors = |state: &mut Plot3DState, data: &Surface3DData, map: ColorMap| {
        let scale = ColorScale::new([0.0, 1.0]).unwrap().color_map(map);
        overlays.colors(&ctx, state, Surface3D::new(data).color_scale(scale).into())
    };
    let mut state = Plot3DState::default();
    colors(&mut state, &surface, ColorMap::Viridis);
    let before = colors(&mut state, &surface, ColorMap::Viridis);
    assert!(!before.is_empty());
    let warm = colors(&mut state, &surface, ColorMap::Gray);
    assert!(before != warm, "warm cache kept old colormap");
    assert!(warm == colors(&mut Plot3DState::default(), &surface, ColorMap::Gray));

    let recolored = surface.clone().with_scalars(&[0.25; 9]).unwrap();
    let updated = colors(&mut state, &recolored, ColorMap::Gray);
    assert!(warm != updated, "warm cache kept old scalars");
    assert!(updated == colors(&mut Plot3DState::default(), &recolored, ColorMap::Gray));
}

#[test]
fn cached_mesh_tracks_wireframe_and_in_place_reference_plane_changes() {
    let mesh = TriMesh3DData::new(
        vec![[0.0; 3], [1.0, 0.0, 0.6], [0.0, 1.0, 0.8]],
        vec![[0, 1, 2]],
    )
    .unwrap();
    let ctx = Context::default();
    let mut state = Plot3DState::default();
    let mut overlays = Overlays {
        planes: [ConstantPlane3D::z_plane(0.4).fill(Color32::GREEN)].into(),
        ..Default::default()
    };
    let solid = TriSurf3D::new(&mesh).into();
    let wire = TriSurf3D::wireframe(&mesh).into();
    let before = overlays.colors(&ctx, &mut state, solid);
    let after = overlays.colors(&ctx, &mut state, wire);
    assert!(before != after);
    assert!(after == overlays.colors(&ctx, &mut Plot3DState::default(), wire));

    Arc::get_mut(&mut overlays.planes).unwrap()[0].fill = Color32::RED;
    let updated = overlays.colors(&ctx, &mut state, wire);
    assert!(after != updated);
    assert!(updated == overlays.colors(&ctx, &mut Plot3DState::default(), wire));
}

#[derive(Default)]
struct Overlays {
    planes: Arc<[ConstantPlane3D]>,
    texts: Arc<[Text3D]>,
}

impl Overlays {
    fn colors(&self, ctx: &Context, state: &mut Plot3DState, layer: Layer3D<'_>) -> Vec<Color32> {
        let frame = ctx.run_ui(support::input(vec![]), |ui| {
            Plot3D::new("cached")
                .depth_mode(DepthMode::ZBuffer)
                .constant_planes(self.planes.clone())
                .texts(self.texts.clone())
                .height(260.0)
                .show(ui, state, &[layer]);
        });
        let mut colors = Vec::new();
        for item in &frame.shapes {
            if let Shape::Mesh(mesh) = &item.shape {
                colors.extend(mesh.vertices.iter().map(|v| v.color));
            }
        }
        frame.drop_without_applying_deltas();
        colors
    }
}
