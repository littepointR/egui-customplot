use egui::{Color32, IdSalt, Stroke};

use super::surfaces::SurfaceShading;
use crate::{
    ColorScale, Contour3DData, ImplicitSurface3DData, SeriesStyle, Surface3DData, TriMesh3DData,
};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum MeshSurfaceStyle {
    #[default]
    Grid,
    CurtainZ,
    Waterfall,
}

/// Wireframe 3D surface mesh (`mesh`, `meshc`, `meshz`, `waterfall`, `fmesh`, `ezmesh`, `ezmeshc`).
#[derive(Clone, Copy, Debug)]
pub struct MeshSurface3D<'a> {
    pub data: &'a Surface3DData,
    pub mesh_style: MeshSurfaceStyle,
    pub name: &'a str,
    pub id_salt: Option<IdSalt>,
    pub hidden_line: bool,
    pub color_scale: Option<ColorScale>,
    pub floor_contours: bool,
    pub floor_z: Option<f64>,
    pub base_z: Option<f64>,
    pub style: SeriesStyle,
}

impl<'a> MeshSurface3D<'a> {
    pub fn new(data: &'a Surface3DData) -> Self {
        Self {
            data,
            mesh_style: MeshSurfaceStyle::Grid,
            name: "",
            id_salt: None,
            hidden_line: true,
            color_scale: None,
            floor_contours: false,
            floor_z: None,
            base_z: None,
            style: SeriesStyle {
                line_width: Some(1.1),
                ..Default::default()
            },
        }
    }
    pub fn curtain(data: &'a Surface3DData) -> Self {
        let mut s = Self::new(data);
        s.mesh_style = MeshSurfaceStyle::CurtainZ;
        s
    }
    pub fn waterfall(data: &'a Surface3DData) -> Self {
        let mut s = Self::new(data);
        s.mesh_style = MeshSurfaceStyle::Waterfall;
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
    pub fn hidden_line(mut self, hidden: bool) -> Self {
        self.hidden_line = hidden;
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
    pub fn width(mut self, width: f32) -> Self {
        self.style.line_width = Some(width);
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
    pub fn base_z(mut self, z: f64) -> Self {
        if z.is_finite() {
            self.base_z = Some(z);
        }
        self
    }
}

/// 3D isolines positioned at each level's `z = level` height (`contour3`, `ML-053`).
#[derive(Clone, Copy, Debug)]
pub struct Contour3D<'a> {
    pub data: &'a Contour3DData,
    pub name: &'a str,
    pub id_salt: Option<IdSalt>,
    pub color_scale: Option<ColorScale>,
    pub style: SeriesStyle,
}

impl<'a> Contour3D<'a> {
    pub fn new(data: &'a Contour3DData) -> Self {
        Self {
            data,
            name: "",
            id_salt: None,
            color_scale: None,
            style: SeriesStyle {
                line_width: Some(1.4),
                ..Default::default()
            },
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
    pub fn color(mut self, color: Color32) -> Self {
        self.style.color = Some(color);
        self
    }
    pub fn color_scale(mut self, scale: ColorScale) -> Self {
        self.color_scale = Some(scale);
        self
    }
    pub fn width(mut self, width: f32) -> Self {
        self.style.line_width = Some(width);
        self
    }
}

/// Indexed 3D triangular surface / wireframe / implicit zero level set (`trisurf`, `trimesh` 3D, `fimplicit3`).
#[derive(Clone, Copy, Debug)]
pub struct TriSurf3D<'a> {
    pub data: Option<&'a TriMesh3DData>,
    pub implicit: Option<&'a ImplicitSurface3DData>,
    pub name: &'a str,
    pub id_salt: Option<IdSalt>,
    pub shading: SurfaceShading,
    pub wireframe: bool,
    pub hidden_line: bool,
    pub color_scale: Option<ColorScale>,
    pub edge_stroke: Option<Stroke>,
    pub alpha: f32,
    pub style: SeriesStyle,
}

impl<'a> TriSurf3D<'a> {
    pub fn new(data: &'a TriMesh3DData) -> Self {
        Self {
            data: Some(data),
            implicit: None,
            name: "",
            id_salt: None,
            shading: SurfaceShading::Faceted,
            wireframe: false,
            hidden_line: true,
            color_scale: None,
            edge_stroke: Some(Stroke::new(0.6, Color32::from_black_alpha(120))),
            alpha: 0.96,
            style: SeriesStyle::default(),
        }
    }
    pub fn wireframe(data: &'a TriMesh3DData) -> Self {
        let mut s = Self::new(data);
        s.wireframe = true;
        s
    }
    pub fn from_implicit(implicit: &'a ImplicitSurface3DData) -> Self {
        let mut s = Self {
            data: implicit.mesh(),
            implicit: Some(implicit),
            name: "",
            id_salt: None,
            shading: SurfaceShading::Interp,
            wireframe: false,
            hidden_line: true,
            color_scale: None,
            edge_stroke: None,
            alpha: 0.96,
            style: SeriesStyle::default(),
        };
        s.style.color = Some(Color32::from_rgb(76, 148, 214));
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
    pub fn edge_stroke(mut self, stroke: impl Into<Option<Stroke>>) -> Self {
        self.edge_stroke = stroke.into();
        self
    }
    pub fn hidden_line(mut self, hidden: bool) -> Self {
        self.hidden_line = hidden;
        self
    }
    pub fn alpha(mut self, alpha: f32) -> Self {
        if alpha.is_finite() {
            self.alpha = alpha.clamp(0.0, 1.0);
        }
        self
    }
}
