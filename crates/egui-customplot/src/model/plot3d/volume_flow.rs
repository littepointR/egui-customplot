use egui::{Color32, IdSalt, Stroke};

use super::surfaces::SurfaceShading;
use crate::{
    ColorScale, ConePlot3DData, Isocaps3DData, Isosurface3DData, MarkerShape, MarkerStyle,
    SeriesStyle, StreamParticles3DData, StreamParticlesPlayback, StreamRibbon3DData,
    StreamTube3DData, Streamline3DData, TetraMesh3DData, VolumeSliceData,
};

macro_rules! common_3d_builders {
    ($t:ty) => {
        impl<'a> $t {
            pub fn name(mut self, name: &'a str) -> Self {
                self.name = name;
                self
            }
            pub fn id_salt(mut self, id: impl egui::AsIdSalt) -> Self {
                self.id_salt = Some(IdSalt::new(id));
                self
            }
            pub fn color_scale(mut self, scale: ColorScale) -> Self {
                self.color_scale = Some(scale);
                self
            }
        }
    };
}

macro_rules! color_alpha_shading_builders {
    ($t:ty) => {
        common_3d_builders!($t);
        impl<'a> $t {
            pub fn shading(mut self, shading: SurfaceShading) -> Self {
                self.shading = shading;
                self
            }
            pub fn color(mut self, color: Color32) -> Self {
                self.style.color = Some(color);
                self
            }
            pub fn alpha(mut self, alpha: f32) -> Self {
                if alpha.is_finite() {
                    self.alpha = alpha.clamp(0.0, 1.0);
                }
                self
            }
        }
    };
}

/// 3D volumetric orthogonal/arbitrary slice layer (`slice`, `ML-077`).
#[derive(Clone, Copy, Debug)]
pub struct VolumeSlice3D<'a> {
    pub data: &'a VolumeSliceData,
    pub name: &'a str,
    pub id_salt: Option<IdSalt>,
    pub shading: SurfaceShading,
    pub color_scale: Option<ColorScale>,
    pub wireframe_stroke: Option<Stroke>,
    pub alpha: f32,
    pub style: SeriesStyle,
}

impl<'a> VolumeSlice3D<'a> {
    pub fn new(data: &'a VolumeSliceData) -> Self {
        Self {
            data,
            name: "",
            id_salt: None,
            shading: SurfaceShading::Interp,
            color_scale: None,
            wireframe_stroke: None,
            alpha: 0.94,
            style: SeriesStyle::default(),
        }
    }
    pub fn shading(mut self, shading: SurfaceShading) -> Self {
        self.shading = shading;
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
}
common_3d_builders!(VolumeSlice3D<'a>);

/// Volumetric iso-surface mesh layer (`isosurface`, `EX-023`).
#[derive(Clone, Copy, Debug)]
pub struct Isosurface3D<'a> {
    pub data: &'a Isosurface3DData,
    pub name: &'a str,
    pub id_salt: Option<IdSalt>,
    pub shading: SurfaceShading,
    pub color_scale: Option<ColorScale>,
    pub alpha: f32,
    pub style: SeriesStyle,
}

impl<'a> Isosurface3D<'a> {
    pub fn new(data: &'a Isosurface3DData) -> Self {
        Self {
            data,
            name: "",
            id_salt: None,
            shading: SurfaceShading::Interp,
            color_scale: None,
            alpha: 0.95,
            style: SeriesStyle {
                color: Some(Color32::from_rgb(76, 148, 214)),
                ..Default::default()
            },
        }
    }
}
color_alpha_shading_builders!(Isosurface3D<'a>);

/// Volumetric end-cap mesh layer (`isocaps`, `EX-024`).
#[derive(Clone, Copy, Debug)]
pub struct Isocaps3D<'a> {
    pub data: &'a Isocaps3DData,
    pub name: &'a str,
    pub id_salt: Option<IdSalt>,
    pub shading: SurfaceShading,
    pub color_scale: Option<ColorScale>,
    pub alpha: f32,
    pub style: SeriesStyle,
}

impl<'a> Isocaps3D<'a> {
    pub fn new(data: &'a Isocaps3DData) -> Self {
        Self {
            data,
            name: "",
            id_salt: None,
            shading: SurfaceShading::Interp,
            color_scale: None,
            alpha: 0.98,
            style: SeriesStyle::default(),
        }
    }
}
color_alpha_shading_builders!(Isocaps3D<'a>);

/// 3D tetrahedral volume mesh layer (`tetramesh`, `EX-014`).
#[derive(Clone, Copy, Debug)]
pub struct TetraMesh3D<'a> {
    pub data: &'a TetraMesh3DData,
    pub name: &'a str,
    pub id_salt: Option<IdSalt>,
    pub boundary_only: bool,
    pub color_scale: Option<ColorScale>,
    pub edge_stroke: Option<Stroke>,
    pub alpha: f32,
    pub style: SeriesStyle,
}

impl<'a> TetraMesh3D<'a> {
    pub fn new(data: &'a TetraMesh3DData) -> Self {
        Self {
            data,
            name: "",
            id_salt: None,
            boundary_only: true,
            color_scale: None,
            edge_stroke: Some(Stroke::new(0.8, Color32::from_black_alpha(150))),
            alpha: 0.72,
            style: SeriesStyle::default(),
        }
    }
    pub fn boundary_only(mut self, boundary_only: bool) -> Self {
        self.boundary_only = boundary_only;
        self
    }
    pub fn color(mut self, color: Color32) -> Self {
        self.style.color = Some(color);
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
common_3d_builders!(TetraMesh3D<'a>);

/// 3D streamline layer (`streamline` / `streamslice` 3D).
#[derive(Clone, Copy, Debug)]
pub struct Streamline3D<'a> {
    pub data: &'a Streamline3DData,
    pub name: &'a str,
    pub id_salt: Option<IdSalt>,
    pub color_scale: Option<ColorScale>,
    pub style: SeriesStyle,
}

impl<'a> Streamline3D<'a> {
    pub fn new(data: &'a Streamline3DData) -> Self {
        Self {
            data,
            name: "",
            id_salt: None,
            color_scale: None,
            style: SeriesStyle {
                line_width: Some(1.5),
                ..Default::default()
            },
        }
    }
    pub fn color(mut self, color: Color32) -> Self {
        self.style.color = Some(color);
        self
    }
    pub fn width(mut self, width: f32) -> Self {
        if width.is_finite() && width > 0.0 {
            self.style.line_width = Some(width);
        }
        self
    }
}
common_3d_builders!(Streamline3D<'a>);

/// 3D moving streamline particles (`streamparticles`, `ML-073`).
#[derive(Clone, Copy, Debug)]
pub struct StreamParticles3D<'a> {
    pub data: &'a StreamParticles3DData,
    pub playback: &'a StreamParticlesPlayback,
    pub name: &'a str,
    pub id_salt: Option<IdSalt>,
    pub color_scale: Option<ColorScale>,
    pub show_lines: bool,
    pub style: SeriesStyle,
}

impl<'a> StreamParticles3D<'a> {
    pub fn new(data: &'a StreamParticles3DData, playback: &'a StreamParticlesPlayback) -> Self {
        Self {
            data,
            playback,
            name: "",
            id_salt: None,
            color_scale: None,
            show_lines: true,
            style: SeriesStyle {
                marker_shape: Some(MarkerShape::Circle),
                marker_radius: Some(4.0),
                ..Default::default()
            },
        }
    }
    pub fn marker(mut self, marker: MarkerStyle) -> Self {
        self.style.marker_shape = Some(marker.shape);
        self.style.marker_radius = Some(marker.radius);
        self
    }
    pub fn color(mut self, color: Color32) -> Self {
        self.style.color = Some(color);
        self
    }
    pub fn show_lines(mut self, show: bool) -> Self {
        self.show_lines = show;
        self
    }
}
common_3d_builders!(StreamParticles3D<'a>);

/// 3D curl-twisted ribbon flow layer (`streamribbon`, `ML-074`).
#[derive(Clone, Copy, Debug)]
pub struct StreamRibbon3D<'a> {
    pub data: &'a StreamRibbon3DData,
    pub name: &'a str,
    pub id_salt: Option<IdSalt>,
    pub shading: SurfaceShading,
    pub color_scale: Option<ColorScale>,
    pub edge_stroke: Option<Stroke>,
    pub alpha: f32,
    pub style: SeriesStyle,
}

impl<'a> StreamRibbon3D<'a> {
    pub fn new(data: &'a StreamRibbon3DData) -> Self {
        Self {
            data,
            name: "",
            id_salt: None,
            shading: SurfaceShading::Flat,
            color_scale: None,
            edge_stroke: None,
            alpha: 0.94,
            style: SeriesStyle::default(),
        }
    }
    pub fn edge_stroke(mut self, stroke: impl Into<Option<Stroke>>) -> Self {
        self.edge_stroke = stroke.into();
        self
    }
}
color_alpha_shading_builders!(StreamRibbon3D<'a>);

/// 3D divergence-modulated stream tube layer (`streamtube`, `ML-075`).
#[derive(Clone, Copy, Debug)]
pub struct StreamTube3D<'a> {
    pub data: &'a StreamTube3DData,
    pub name: &'a str,
    pub id_salt: Option<IdSalt>,
    pub shading: SurfaceShading,
    pub color_scale: Option<ColorScale>,
    pub edge_stroke: Option<Stroke>,
    pub alpha: f32,
    pub style: SeriesStyle,
}

impl<'a> StreamTube3D<'a> {
    pub fn new(data: &'a StreamTube3DData) -> Self {
        Self {
            data,
            name: "",
            id_salt: None,
            shading: SurfaceShading::Interp,
            color_scale: None,
            edge_stroke: None,
            alpha: 0.95,
            style: SeriesStyle::default(),
        }
    }
    pub fn edge_stroke(mut self, stroke: impl Into<Option<Stroke>>) -> Self {
        self.edge_stroke = stroke.into();
        self
    }
}
color_alpha_shading_builders!(StreamTube3D<'a>);

/// 3D cone vector field layer (`coneplot`, `ML-076`).
#[derive(Clone, Copy, Debug)]
pub struct ConePlot3D<'a> {
    pub data: &'a ConePlot3DData,
    pub name: &'a str,
    pub id_salt: Option<IdSalt>,
    pub shading: SurfaceShading,
    pub color_scale: Option<ColorScale>,
    pub alpha: f32,
    pub style: SeriesStyle,
}

impl<'a> ConePlot3D<'a> {
    pub fn new(data: &'a ConePlot3DData) -> Self {
        Self {
            data,
            name: "",
            id_salt: None,
            shading: SurfaceShading::Flat,
            color_scale: None,
            alpha: 0.96,
            style: SeriesStyle::default(),
        }
    }
}
color_alpha_shading_builders!(ConePlot3D<'a>);
