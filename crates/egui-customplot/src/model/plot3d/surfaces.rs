use egui::{Color32, IdSalt, Stroke};

use crate::{ColorScale, Ribbon3DData, SeriesStyle, Surface3DData};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SurfaceShading {
    #[default]
    Faceted,
    Flat,
    Interp,
}

/// Directional specular+diffuse lighting configuration for 3D surfaces (`light` / `surfl`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LightSource3D {
    pub azimuth_deg: f64,
    pub elevation_deg: f64,
    pub ambient: f32,
    pub diffuse: f32,
    pub specular: f32,
    pub shininess: f32,
}

impl Default for LightSource3D {
    fn default() -> Self {
        Self {
            azimuth_deg: -45.0,
            elevation_deg: 45.0,
            ambient: 0.35,
            diffuse: 0.55,
            specular: 0.25,
            shininess: 16.0,
        }
    }
}

impl LightSource3D {
    pub fn direction(self) -> [f64; 3] {
        let az = self.azimuth_deg.to_radians();
        let el = self.elevation_deg.clamp(-90.0, 90.0).to_radians();
        let (se, ce) = el.sin_cos();
        let (sa, ca) = az.sin_cos();
        [ce * ca, ce * sa, se]
    }
}

/// Filled 3D parametric/grid surface (`surf`, `surfc`, `surfl`, `fsurf`, `ezsurf*`, `surface`, `surfnorm`, `sphere`, `cylinder`, `ellipsoid`).
#[derive(Clone, Copy, Debug)]
pub struct Surface3D<'a> {
    pub data: &'a Surface3DData,
    pub name: &'a str,
    pub id_salt: Option<IdSalt>,
    pub shading: SurfaceShading,
    pub color_scale: Option<ColorScale>,
    pub light: Option<LightSource3D>,
    pub colormap_lighting: bool,
    pub wireframe_stroke: Option<Stroke>,
    pub alpha: f32,
    pub primitive: bool,
    pub floor_contours: bool,
    pub floor_z: Option<f64>,
    pub show_normals: bool,
    pub normal_scale: f64,
    pub normal_stride: usize,
    pub style: SeriesStyle,
}

impl<'a> Surface3D<'a> {
    pub fn new(data: &'a Surface3DData) -> Self {
        Self {
            data,
            name: "",
            id_salt: None,
            shading: SurfaceShading::Faceted,
            color_scale: None,
            light: Some(LightSource3D::default()),
            colormap_lighting: false,
            wireframe_stroke: Some(Stroke::new(0.6, Color32::from_black_alpha(120))),
            alpha: 0.96,
            primitive: false,
            floor_contours: false,
            floor_z: None,
            show_normals: false,
            normal_scale: 0.25,
            normal_stride: 2,
            style: SeriesStyle::default(),
        }
    }

    pub fn primitive(data: &'a Surface3DData) -> Self {
        let mut s = Self::new(data);
        s.primitive = true;
        s
    }
    pub fn name(mut self, name: &'a str) -> Self {
        self.name = name;
        self
    }
    pub fn id_salt(mut self, id: impl egui::AsIdSalt) -> Self {
        self.id_salt = Some(IdSalt::new(id));
        self
    }
    pub fn shading(mut self, shading: SurfaceShading) -> Self {
        self.shading = shading;
        self
    }
    pub fn color(mut self, color: Color32) -> Self {
        self.style.color = Some(color);
        self
    }
    pub fn color_scale(mut self, scale: ColorScale) -> Self {
        self.color_scale = Some(scale);
        self
    }
    pub fn light(mut self, light: Option<LightSource3D>) -> Self {
        self.light = light;
        self
    }
    pub fn colormap_lighting(mut self, enabled: bool) -> Self {
        self.colormap_lighting = enabled;
        self
    }
    pub fn wireframe_stroke(mut self, stroke: impl Into<Option<Stroke>>) -> Self {
        self.wireframe_stroke = stroke.into();
        self
    }
    pub fn alpha(mut self, alpha: f32) -> Self {
        if alpha.is_finite() {
            self.alpha = alpha.clamp(0.0, 1.0);
        }
        self
    }
    pub fn floor_contours(mut self, show: bool) -> Self {
        self.floor_contours = show;
        self
    }
    pub fn floor_z(mut self, z: f64) -> Self {
        if z.is_finite() {
            self.floor_z = Some(z);
        }
        self
    }
    pub fn show_normals(mut self, show: bool, scale: f64) -> Self {
        self.show_normals = show;
        if scale.is_finite() && scale > 0.0 {
            self.normal_scale = scale;
        }
        self
    }
    pub fn normal_stride(mut self, stride: usize) -> Self {
        self.normal_stride = stride.max(1);
        self
    }
}

/// 3D ribbon strip plot (`ribbon`, `ML-062`).
#[derive(Clone, Copy, Debug)]
pub struct Ribbon3D<'a> {
    pub data: &'a Ribbon3DData,
    pub name: &'a str,
    pub id_salt: Option<IdSalt>,
    pub shading: SurfaceShading,
    pub color_scale: Option<ColorScale>,
    pub edge_stroke: Option<Stroke>,
    pub alpha: f32,
    pub style: SeriesStyle,
}

impl<'a> Ribbon3D<'a> {
    pub fn new(data: &'a Ribbon3DData) -> Self {
        Self {
            data,
            name: "",
            id_salt: None,
            shading: SurfaceShading::Flat,
            color_scale: None,
            edge_stroke: Some(Stroke::new(0.7, Color32::from_black_alpha(120))),
            alpha: 0.92,
            style: SeriesStyle::default(),
        }
    }
    pub fn name(mut self, name: &'a str) -> Self {
        self.name = name;
        self
    }
    pub fn id_salt(mut self, id: impl egui::AsIdSalt) -> Self {
        self.id_salt = Some(IdSalt::new(id));
        self
    }
    pub fn shading(mut self, shading: SurfaceShading) -> Self {
        self.shading = shading;
        self
    }
    pub fn color_scale(mut self, scale: ColorScale) -> Self {
        self.color_scale = Some(scale);
        self
    }
    pub fn edge_stroke(mut self, stroke: impl Into<Option<Stroke>>) -> Self {
        self.edge_stroke = stroke.into();
        self
    }
    pub fn alpha(mut self, alpha: f32) -> Self {
        if alpha.is_finite() {
            self.alpha = alpha.clamp(0.0, 1.0);
        }
        self
    }
}
