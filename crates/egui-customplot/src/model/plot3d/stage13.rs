use egui::{Color32, IdSalt, Stroke};

use super::surfaces::{LightSource3D, SurfaceShading};
use crate::{AlphaShape3DData, ColorScale, Graph3DData, MarkerShape, Patch3DData, SeriesStyle};

/// General 3D polygon patch layer (`patch`, `EXT-09`).
#[derive(Clone, Copy, Debug)]
pub struct Patch3D<'a> {
    pub data: &'a Patch3DData,
    pub shading: SurfaceShading,
    pub color_scale: Option<ColorScale>,
    pub light: LightSource3D,
    pub alpha: f32,
    pub edge_stroke: Option<Stroke>,
    pub name: &'a str,
    pub id_salt: Option<IdSalt>,
    pub style: SeriesStyle,
}

impl<'a> Patch3D<'a> {
    pub fn new(data: &'a Patch3DData) -> Self {
        Self {
            data,
            shading: SurfaceShading::Faceted,
            color_scale: None,
            light: LightSource3D::default(),
            alpha: 0.96,
            edge_stroke: Some(Stroke::new(0.8, Color32::from_black_alpha(150))),
            name: "",
            id_salt: None,
            style: SeriesStyle::default(),
        }
    }
    pub fn shading(mut self, shading: SurfaceShading) -> Self {
        self.shading = shading;
        self
    }
    pub fn color_scale(mut self, scale: ColorScale) -> Self {
        self.color_scale = Some(scale);
        self
    }
    pub fn light(mut self, light: LightSource3D) -> Self {
        self.light = light;
        self
    }
    pub fn specular(mut self, shininess: f32, strength: f32) -> Self {
        if shininess.is_finite() && shininess > 0.0 {
            self.light.shininess = shininess;
        }
        if strength.is_finite() && strength >= 0.0 {
            self.light.specular = strength;
        }
        self
    }
    pub fn alpha(mut self, alpha: f32) -> Self {
        if alpha.is_finite() {
            self.alpha = alpha.clamp(0.0, 1.0);
        }
        self
    }
    pub fn edge_stroke(mut self, stroke: Option<Stroke>) -> Self {
        self.edge_stroke = stroke;
        self
    }
    pub fn color(mut self, color: Color32) -> Self {
        self.style.color = Some(color);
        self
    }
    pub fn name(mut self, name: &'a str) -> Self {
        self.name = name;
        self
    }
    pub fn id_salt(mut self, id: impl egui::AsIdSalt) -> Self {
        self.id_salt = Some(IdSalt::new(id));
        self
    }
}

/// 3D alpha-shape concave shell layer (`plot(alphaShape3D)`, `EXT-10`).
#[derive(Clone, Copy, Debug)]
pub struct AlphaShape3D<'a> {
    pub data: &'a AlphaShape3DData,
    pub shading: SurfaceShading,
    pub color_scale: Option<ColorScale>,
    pub light: LightSource3D,
    pub alpha: f32,
    pub edge_stroke: Option<Stroke>,
    pub name: &'a str,
    pub id_salt: Option<IdSalt>,
    pub style: SeriesStyle,
}

impl<'a> AlphaShape3D<'a> {
    pub fn new(data: &'a AlphaShape3DData) -> Self {
        Self {
            data,
            shading: SurfaceShading::Faceted,
            color_scale: None,
            light: LightSource3D::default(),
            alpha: 0.90,
            edge_stroke: Some(Stroke::new(0.7, Color32::from_black_alpha(140))),
            name: "",
            id_salt: None,
            style: SeriesStyle::default(),
        }
    }
    pub fn shading(mut self, shading: SurfaceShading) -> Self {
        self.shading = shading;
        self
    }
    pub fn color_scale(mut self, scale: ColorScale) -> Self {
        self.color_scale = Some(scale);
        self
    }
    pub fn light(mut self, light: LightSource3D) -> Self {
        self.light = light;
        self
    }
    pub fn alpha(mut self, alpha: f32) -> Self {
        if alpha.is_finite() {
            self.alpha = alpha.clamp(0.0, 1.0);
        }
        self
    }
    pub fn edge_stroke(mut self, stroke: Option<Stroke>) -> Self {
        self.edge_stroke = stroke;
        self
    }
    pub fn color(mut self, color: Color32) -> Self {
        self.style.color = Some(color);
        self
    }
    pub fn name(mut self, name: &'a str) -> Self {
        self.name = name;
        self
    }
    pub fn id_salt(mut self, id: impl egui::AsIdSalt) -> Self {
        self.id_salt = Some(IdSalt::new(id));
        self
    }
}

/// 3D graph and directed network layer (`plot3(graph/digraph)`, `EXT-11`).
#[derive(Clone, Copy, Debug)]
pub struct Graph3D<'a> {
    pub data: &'a Graph3DData,
    pub color_scale: Option<ColorScale>,
    pub edge_stroke: Stroke,
    pub arrow_size: f32,
    pub show_node_labels: bool,
    pub show_edge_labels: bool,
    pub name: &'a str,
    pub id_salt: Option<IdSalt>,
    pub style: SeriesStyle,
}

impl<'a> Graph3D<'a> {
    pub fn new(data: &'a Graph3DData) -> Self {
        Self {
            data,
            color_scale: None,
            edge_stroke: Stroke::new(1.2, Color32::from_rgb(95, 115, 145)),
            arrow_size: 7.0,
            show_node_labels: false,
            show_edge_labels: false,
            name: "",
            id_salt: None,
            style: SeriesStyle {
                marker_shape: Some(MarkerShape::Circle),
                marker_radius: Some(4.5),
                ..Default::default()
            },
        }
    }
    pub fn node_shape(mut self, shape: MarkerShape) -> Self {
        self.style.marker_shape = Some(shape);
        self
    }
    pub fn node_radius(mut self, radius: f32) -> Self {
        if radius.is_finite() && radius > 0.0 {
            self.style.marker_radius = Some(radius);
        }
        self
    }
    pub fn node_color(mut self, color: Color32) -> Self {
        self.style.color = Some(color);
        self
    }
    pub fn color_scale(mut self, scale: ColorScale) -> Self {
        self.color_scale = Some(scale);
        self
    }
    pub fn edge_stroke(mut self, stroke: Stroke) -> Self {
        self.edge_stroke = stroke;
        self
    }
    pub fn arrow_size(mut self, px: f32) -> Self {
        if px.is_finite() && px > 0.0 {
            self.arrow_size = px;
        }
        self
    }
    pub fn show_node_labels(mut self, show: bool) -> Self {
        self.show_node_labels = show;
        self
    }
    pub fn show_edge_labels(mut self, show: bool) -> Self {
        self.show_edge_labels = show;
        self
    }
    pub fn name(mut self, name: &'a str) -> Self {
        self.name = name;
        self
    }
    pub fn id_salt(mut self, id: impl egui::AsIdSalt) -> Self {
        self.id_salt = Some(IdSalt::new(id));
        self
    }
}
