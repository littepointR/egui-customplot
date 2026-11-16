use egui::{Color32, IdSalt, Stroke};

use crate::{
    Bar3DData, ColorScale, MarkerShape, MarkerStyle, Pie3DData, Polygon3DData, Quiver3DData,
    SeriesStyle, Stem3DData,
};

/// 3D vertical/horizontal bars (`bar3` / `bar3h`, `ML-036` / `ML-037`).
#[derive(Clone, Copy, Debug)]
pub struct Bar3D<'a> {
    pub data: &'a Bar3DData,
    pub name: &'a str,
    pub id_salt: Option<IdSalt>,
    pub color_scale: Option<ColorScale>,
    pub edge_stroke: Option<Stroke>,
    pub alpha: f32,
    pub style: SeriesStyle,
}

impl<'a> Bar3D<'a> {
    pub fn new(data: &'a Bar3DData) -> Self {
        Self {
            data,
            name: "",
            id_salt: None,
            color_scale: None,
            edge_stroke: Some(Stroke::new(0.8, Color32::from_black_alpha(110))),
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
    pub fn color(mut self, color: Color32) -> Self {
        self.style.color = Some(color);
        self
    }
    pub fn color_scale(mut self, scale: ColorScale) -> Self {
        self.color_scale = Some(scale);
        self
    }
    pub fn edge_stroke(mut self, stroke: Option<Stroke>) -> Self {
        self.edge_stroke = stroke;
        self
    }
    pub fn alpha(mut self, alpha: f32) -> Self {
        if alpha.is_finite() {
            self.alpha = alpha.clamp(0.0, 1.0);
        }
        self
    }
}

/// 3D stems rooted on `z = base_z` (`stem3`, `ML-040`).
#[derive(Clone, Copy, Debug)]
pub struct Stem3D<'a> {
    pub data: &'a Stem3DData,
    pub name: &'a str,
    pub id_salt: Option<IdSalt>,
    pub show_base: bool,
    pub style: SeriesStyle,
}

impl<'a> Stem3D<'a> {
    pub fn new(data: &'a Stem3DData) -> Self {
        Self {
            data,
            name: "",
            id_salt: None,
            show_base: true,
            style: SeriesStyle {
                marker_shape: Some(MarkerShape::Circle),
                marker_radius: Some(3.8),
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
    pub fn show_base(mut self, show: bool) -> Self {
        self.show_base = show;
        self
    }
    pub fn marker(mut self, marker: MarkerStyle) -> Self {
        self.style.marker_shape = Some(marker.shape);
        self.style.marker_radius = Some(marker.radius);
        self
    }
}

/// Extruded 3D pie chart (`pie3`, `LG-002`).
#[derive(Clone, Copy, Debug)]
pub struct Pie3D<'a> {
    pub data: &'a Pie3DData,
    pub name: &'a str,
    pub id_salt: Option<IdSalt>,
    pub show_labels: bool,
    pub border: Option<Stroke>,
    pub style: SeriesStyle,
}

impl<'a> Pie3D<'a> {
    pub fn new(data: &'a Pie3DData) -> Self {
        Self {
            data,
            name: "",
            id_salt: None,
            show_labels: true,
            border: Some(Stroke::new(0.9, Color32::from_black_alpha(120))),
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
    pub fn show_labels(mut self, show: bool) -> Self {
        self.show_labels = show;
        self
    }
    pub fn border(mut self, stroke: Option<Stroke>) -> Self {
        self.border = stroke;
        self
    }
}

/// 3D quiver arrows (`quiver3`, `ML-057`).
#[derive(Clone, Copy, Debug)]
pub struct Quiver3D<'a> {
    pub data: &'a Quiver3DData,
    pub name: &'a str,
    pub id_salt: Option<IdSalt>,
    pub head_size: f32,
    pub show_heads: bool,
    pub style: SeriesStyle,
}

impl<'a> Quiver3D<'a> {
    pub fn new(data: &'a Quiver3DData) -> Self {
        Self {
            data,
            name: "",
            id_salt: None,
            head_size: 8.0,
            show_heads: true,
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
    pub fn color(mut self, color: Color32) -> Self {
        self.style.color = Some(color);
        self
    }
    pub fn width(mut self, width: f32) -> Self {
        self.style.line_width = Some(width);
        self
    }
    pub fn head_size(mut self, size: f32) -> Self {
        if size.is_finite() && size > 0.0 {
            self.head_size = size.clamp(2.0, 24.0);
        }
        self
    }
    pub fn show_heads(mut self, show: bool) -> Self {
        self.show_heads = show;
        self
    }
}

/// 3D filled polygons (`fill3`, `EX-003`).
#[derive(Clone, Copy, Debug)]
pub struct Fill3D<'a> {
    pub data: &'a Polygon3DData,
    pub name: &'a str,
    pub id_salt: Option<IdSalt>,
    pub border: Option<Stroke>,
    pub alpha: f32,
    pub face_colors: Option<&'a [Color32]>,
    pub style: SeriesStyle,
}

impl<'a> Fill3D<'a> {
    pub fn new(data: &'a Polygon3DData) -> Self {
        Self {
            data,
            name: "",
            id_salt: None,
            border: Some(Stroke::new(1.0, Color32::from_black_alpha(150))),
            alpha: 0.9,
            face_colors: None,
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
    pub fn color(mut self, color: Color32) -> Self {
        self.style.color = Some(color);
        self
    }
    pub fn face_colors(mut self, colors: &'a [Color32]) -> Self {
        self.face_colors = Some(colors);
        self
    }
    pub fn alpha(mut self, alpha: f32) -> Self {
        if alpha.is_finite() {
            self.alpha = alpha.clamp(0.0, 1.0);
        }
        self
    }
    pub fn border(mut self, stroke: Option<Stroke>) -> Self {
        self.border = stroke;
        self
    }
}
