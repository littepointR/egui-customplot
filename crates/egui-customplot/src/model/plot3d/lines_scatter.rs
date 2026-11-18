use egui::{Color32, IdSalt};

use crate::{
    Bubble3DData, Comet3DPlayback, LinePattern, MarkerShape, MarkerStyle, Point3DData, SeriesStyle,
    Swarm3DData,
};

/// 3D polyline (`plot3` / `fplot3` / `ezplot3` / `animatedline`, `ML-002` / `ML-011` / `LG-009`).
#[derive(Clone, Copy, Debug)]
pub struct Line3D<'a> {
    pub data: &'a Point3DData,
    pub name: &'a str,
    pub id_salt: Option<IdSalt>,
    pub style: SeriesStyle,
    pub show_markers: bool,
    pub primitive: bool,
}

impl<'a> Line3D<'a> {
    pub fn new(data: &'a Point3DData) -> Self {
        Self {
            data,
            name: "",
            id_salt: None,
            style: SeriesStyle::default(),
            show_markers: false,
            primitive: false,
        }
    }
    pub fn primitive(data: &'a Point3DData) -> Self {
        Self::new(data).as_primitive()
    }
    fn as_primitive(mut self) -> Self {
        self.primitive = true;
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
    pub fn marker(mut self, marker: MarkerStyle) -> Self {
        self.style.marker_shape = Some(marker.shape);
        self.style.marker_radius = Some(marker.radius);
        self.show_markers = true;
        self
    }
}

/// 3D scatter plot (`scatter3`, `ML-014`).
#[derive(Clone, Copy, Debug)]
pub struct Scatter3D<'a> {
    pub data: &'a Point3DData,
    pub name: &'a str,
    pub id_salt: Option<IdSalt>,
    pub color_scale: Option<crate::ColorScale>,
    pub style: SeriesStyle,
}

impl<'a> Scatter3D<'a> {
    pub fn new(data: &'a Point3DData) -> Self {
        Self {
            data,
            name: "",
            id_salt: None,
            color_scale: None,
            style: SeriesStyle {
                marker_shape: Some(MarkerShape::Circle),
                marker_radius: Some(4.0),
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
    pub fn color_scale(mut self, scale: crate::ColorScale) -> Self {
        self.color_scale = Some(scale);
        self
    }
    pub fn marker(mut self, marker: MarkerStyle) -> Self {
        self.style.marker_shape = Some(marker.shape);
        self.style.marker_radius = Some(marker.radius);
        self
    }
}

/// 3D bubble chart (`bubblechart3`, `ML-016`).
#[derive(Clone, Copy, Debug)]
pub struct Bubble3D<'a> {
    pub data: &'a Bubble3DData,
    pub name: &'a str,
    pub id_salt: Option<IdSalt>,
    pub color_scale: Option<crate::ColorScale>,
    pub alpha: f32,
    pub border: Option<egui::Stroke>,
    pub style: SeriesStyle,
}

impl<'a> Bubble3D<'a> {
    pub fn new(data: &'a Bubble3DData) -> Self {
        Self {
            data,
            name: "",
            id_salt: None,
            color_scale: None,
            alpha: 0.68,
            border: Some(egui::Stroke::new(1.0, Color32::from_white_alpha(180))),
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
    pub fn color_scale(mut self, scale: crate::ColorScale) -> Self {
        self.color_scale = Some(scale);
        self
    }
    pub fn alpha(mut self, alpha: f32) -> Self {
        if alpha.is_finite() {
            self.alpha = alpha.clamp(0.0, 1.0);
        }
        self
    }
    pub fn border(mut self, border: Option<egui::Stroke>) -> Self {
        self.border = border;
        self
    }
}

/// 3D swarm chart (`swarmchart3`, `ML-018`).
#[derive(Clone, Copy, Debug)]
pub struct Swarm3D<'a> {
    pub data: &'a Swarm3DData,
    pub name: &'a str,
    pub id_salt: Option<IdSalt>,
    pub group_colors: Option<&'a [Color32]>,
    pub style: SeriesStyle,
}

impl<'a> Swarm3D<'a> {
    pub fn new(data: &'a Swarm3DData) -> Self {
        Self {
            data,
            name: "",
            id_salt: None,
            group_colors: None,
            style: SeriesStyle {
                marker_shape: Some(MarkerShape::Circle),
                marker_radius: Some(3.5),
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
    pub fn group_colors(mut self, colors: &'a [Color32]) -> Self {
        self.group_colors = Some(colors);
        self
    }
    pub fn marker(mut self, marker: MarkerStyle) -> Self {
        self.style.marker_shape = Some(marker.shape);
        self.style.marker_radius = Some(marker.radius);
        self
    }
}

/// 3D comet animation layer (`comet3`, `ML-080`).
#[derive(Clone, Copy, Debug)]
pub struct Comet3D<'a> {
    pub data: &'a Point3DData,
    pub playback: &'a Comet3DPlayback,
    pub name: &'a str,
    pub id_salt: Option<IdSalt>,
    pub tail_color: Option<Color32>,
    pub head_radius: f32,
    pub style: SeriesStyle,
}

impl<'a> Comet3D<'a> {
    pub(crate) fn visible_points(&self) -> &'a [crate::Point3D] {
        let points = self.data.points();
        let head = self.playback.progress_points().round() as usize;
        &points[..head.saturating_add(1).min(points.len())]
    }
    pub fn new(data: &'a Point3DData, playback: &'a Comet3DPlayback) -> Self {
        Self {
            data,
            playback,
            name: "",
            id_salt: None,
            tail_color: None,
            head_radius: 5.5,
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
    pub fn tail_color(mut self, color: Color32) -> Self {
        self.tail_color = Some(color);
        self
    }
    pub fn head_radius(mut self, radius: f32) -> Self {
        if radius.is_finite() && radius > 0.0 {
            self.head_radius = radius.clamp(2.0, 24.0);
        }
        self
    }
}
