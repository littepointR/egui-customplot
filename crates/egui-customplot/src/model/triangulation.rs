use egui::{AsIdSalt, Color32, IdSalt, Stroke};

use crate::{
    AlphaShapeData, ColorScale, LinePattern, MarkerShape, SeriesStyle, TriMesh2D, VoronoiData,
    YAxis,
};

/// 2D triangulation / `trimesh` layer (`triplot` / 2D `trimesh`, `EX-011` / `EX-012`).
#[derive(Clone, Copy, Debug)]
pub struct TriPlot<'a> {
    pub(crate) data: &'a TriMesh2D,
    pub(crate) name: &'a str,
    pub(crate) id_salt: Option<IdSalt>,
    pub(crate) y_axis: YAxis,
    pub(crate) style: SeriesStyle,
    pub(crate) show_markers: bool,
    pub(crate) vertex_scalars: Option<(&'a [f64], ColorScale)>,
    pub(crate) face_fill: Option<Color32>,
}

impl<'a> TriPlot<'a> {
    pub fn new(data: &'a TriMesh2D) -> Self {
        Self {
            data,
            name: "",
            id_salt: None,
            y_axis: YAxis::Left,
            style: SeriesStyle::default(),
            show_markers: false,
            vertex_scalars: None,
            face_fill: None,
        }
    }

    pub fn data(self) -> &'a TriMesh2D {
        self.data
    }
    pub fn name(mut self, name: &'a str) -> Self {
        self.name = name;
        self
    }
    pub fn id_salt(mut self, id: impl AsIdSalt) -> Self {
        self.id_salt = Some(IdSalt::new(id));
        self
    }
    pub fn y_axis(mut self, axis: YAxis) -> Self {
        self.y_axis = axis;
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
    pub fn pattern(mut self, pattern: LinePattern) -> Self {
        self.style.line_pattern = Some(pattern);
        self
    }
    pub fn style(mut self, style: SeriesStyle) -> Self {
        self.style = style.over(self.style);
        self
    }
    pub fn markers(mut self, show: bool) -> Self {
        self.show_markers = show;
        self
    }
    /// Color wireframe edges using scalar interpolation per vertex (2D `trimesh`).
    pub fn vertex_scalars(mut self, values: &'a [f64], scale: ColorScale) -> Self {
        self.vertex_scalars = Some((values, scale));
        self
    }
    pub fn face_fill(mut self, fill: Color32) -> Self {
        self.face_fill = Some(fill);
        self
    }
}

/// 2D Voronoi tessellation layer (`voronoi` / `EX-015`).
#[derive(Clone, Copy, Debug)]
pub struct Voronoi<'a> {
    pub(crate) data: &'a VoronoiData,
    pub(crate) name: &'a str,
    pub(crate) id_salt: Option<IdSalt>,
    pub(crate) y_axis: YAxis,
    pub(crate) style: SeriesStyle,
    pub(crate) show_sites: bool,
}

impl<'a> Voronoi<'a> {
    pub fn new(data: &'a VoronoiData) -> Self {
        Self {
            data,
            name: "",
            id_salt: None,
            y_axis: YAxis::Left,
            style: SeriesStyle::default(),
            show_sites: true,
        }
    }

    pub fn data(self) -> &'a VoronoiData {
        self.data
    }
    pub fn name(mut self, name: &'a str) -> Self {
        self.name = name;
        self
    }
    pub fn id_salt(mut self, id: impl AsIdSalt) -> Self {
        self.id_salt = Some(IdSalt::new(id));
        self
    }
    pub fn y_axis(mut self, axis: YAxis) -> Self {
        self.y_axis = axis;
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
    pub fn pattern(mut self, pattern: LinePattern) -> Self {
        self.style.line_pattern = Some(pattern);
        self
    }
    pub fn marker_shape(mut self, shape: MarkerShape) -> Self {
        self.style.marker_shape = Some(shape);
        self
    }
    pub fn marker_radius(mut self, radius: f32) -> Self {
        self.style.marker_radius = Some(radius);
        self
    }
    pub fn style(mut self, style: SeriesStyle) -> Self {
        self.style = style.over(self.style);
        self
    }
    pub fn show_sites(mut self, show: bool) -> Self {
        self.show_sites = show;
        self
    }
}

/// 2D Alpha-shape filled region and extracted boundary (`plot(alphaShape)` / `EX-017`).
#[derive(Clone, Copy, Debug)]
pub struct AlphaShape<'a> {
    pub(crate) data: &'a AlphaShapeData,
    pub(crate) name: &'a str,
    pub(crate) id_salt: Option<IdSalt>,
    pub(crate) y_axis: YAxis,
    pub(crate) style: SeriesStyle,
    pub(crate) border: Stroke,
    pub(crate) show_boundary: bool,
}

impl<'a> AlphaShape<'a> {
    pub fn new(data: &'a AlphaShapeData) -> Self {
        Self {
            data,
            name: "",
            id_salt: None,
            y_axis: YAxis::Left,
            style: SeriesStyle::default(),
            border: Stroke::new(1.5, Color32::from_rgb(20, 40, 80)),
            show_boundary: true,
        }
    }

    pub fn data(self) -> &'a AlphaShapeData {
        self.data
    }
    pub fn name(mut self, name: &'a str) -> Self {
        self.name = name;
        self
    }
    pub fn id_salt(mut self, id: impl AsIdSalt) -> Self {
        self.id_salt = Some(IdSalt::new(id));
        self
    }
    pub fn y_axis(mut self, axis: YAxis) -> Self {
        self.y_axis = axis;
        self
    }
    pub fn fill_color(mut self, color: Color32) -> Self {
        self.style.color = Some(color);
        self
    }
    pub fn border(mut self, border: impl Into<Stroke>) -> Self {
        self.border = border.into();
        self
    }
    pub fn show_boundary(mut self, show: bool) -> Self {
        self.show_boundary = show;
        self
    }
    pub fn style(mut self, style: SeriesStyle) -> Self {
        self.style = style.over(self.style);
        self
    }
}
