use egui::{AsIdSalt, Color32, IdSalt};

use crate::{
    ColorMap, ColorScale, MarkerShape, ScalarVolume3D, SeriesStyle,
    data::{PointCloud3DData, VolumeRenderMode, VolumeTransferFunction},
};

/// Direct volume raycasting layer (`volshow`, `EXT-23`) for [`Plot3D`].
#[derive(Clone, Copy, Debug)]
pub struct VolumeRender3D<'a> {
    pub(crate) volume: &'a ScalarVolume3D,
    pub(crate) transfer: VolumeTransferFunction,
    pub(crate) mode: VolumeRenderMode,
    pub(crate) sample_steps: usize,
    pub(crate) max_resolution: usize,
    pub(crate) shading: bool,
    pub(crate) opacity_scale: f32,
    pub(crate) name: &'a str,
    pub(crate) id_salt: Option<IdSalt>,
}

impl<'a> VolumeRender3D<'a> {
    pub fn new(volume: &'a ScalarVolume3D) -> Self {
        Self {
            volume,
            transfer: VolumeTransferFunction::default(),
            mode: VolumeRenderMode::DirectAlpha,
            sample_steps: 56,
            max_resolution: 120,
            shading: true,
            opacity_scale: 1.0,
            name: "",
            id_salt: None,
        }
    }
    pub fn transfer(mut self, tf: VolumeTransferFunction) -> Self {
        self.transfer = tf;
        self
    }
    pub fn mode(mut self, mode: VolumeRenderMode) -> Self {
        self.mode = mode;
        self
    }
    pub fn sample_steps(mut self, steps: usize) -> Self {
        self.sample_steps = steps.clamp(12, 256);
        self
    }
    pub fn max_resolution(mut self, res: usize) -> Self {
        self.max_resolution = res.clamp(16, 280);
        self
    }
    pub fn shading(mut self, shading: bool) -> Self {
        self.shading = shading;
        self
    }
    pub fn opacity_scale(mut self, scale: f32) -> Self {
        if scale.is_finite() {
            self.opacity_scale = scale.clamp(0.05, 4.0);
        }
        self
    }
    pub fn name(mut self, name: &'a str) -> Self {
        self.name = name;
        self
    }
    pub fn id_salt(mut self, id: impl AsIdSalt) -> Self {
        self.id_salt = Some(IdSalt::new(id));
        self
    }
}

/// High-density octree-LOD 3D point cloud layer (`pcshow`, `EXT-24`) for [`Plot3D`].
#[derive(Clone, Copy, Debug)]
pub struct PointCloud3D<'a> {
    pub(crate) data: &'a PointCloud3DData,
    pub(crate) color_scale: Option<ColorScale>,
    pub(crate) color_map: ColorMap,
    pub(crate) max_points: usize,
    pub(crate) pixel_cell: f32,
    pub(crate) style: SeriesStyle,
    pub(crate) name: &'a str,
    pub(crate) id_salt: Option<IdSalt>,
}

impl<'a> PointCloud3D<'a> {
    pub fn new(data: &'a PointCloud3DData) -> Self {
        let style = SeriesStyle {
            marker_shape: Some(MarkerShape::Circle),
            marker_radius: Some(2.6),
            ..Default::default()
        };
        Self {
            data,
            color_scale: None,
            color_map: ColorMap::Viridis,
            max_points: 20_000,
            pixel_cell: 4.0,
            style,
            name: "",
            id_salt: None,
        }
    }
    pub fn color(mut self, color: Color32) -> Self {
        self.style.color = Some(color);
        self
    }
    pub fn color_map(mut self, map: ColorMap) -> Self {
        self.color_map = map;
        self
    }
    pub fn color_scale(mut self, scale: ColorScale) -> Self {
        self.color_scale = Some(scale);
        self
    }
    pub fn point_size(mut self, radius: f32) -> Self {
        if radius.is_finite() {
            self.style.marker_radius = Some(radius.clamp(0.5, 12.0));
        }
        self
    }
    pub fn max_points(mut self, max_points: usize) -> Self {
        self.max_points = max_points.clamp(128, 250_000);
        self
    }
    pub fn pixel_cell(mut self, px: f32) -> Self {
        if px.is_finite() {
            self.pixel_cell = px.clamp(1.0, 24.0);
        }
        self
    }
    pub fn name(mut self, name: &'a str) -> Self {
        self.name = name;
        self
    }
    pub fn id_salt(mut self, id: impl AsIdSalt) -> Self {
        self.id_salt = Some(IdSalt::new(id));
        self
    }
}
