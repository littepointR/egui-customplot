use egui::{Color32, IdSalt};

use crate::{ColorScale, GraphData, SeriesStyle, YAxis};

#[derive(Clone, Copy, Debug)]
pub struct GraphHighlight<'a> {
    pub nodes: &'a [usize],
    pub edges: &'a [usize],
    pub color: Color32,
}

impl<'a> Default for GraphHighlight<'a> {
    fn default() -> Self {
        Self {
            nodes: &[],
            edges: &[],
            color: Color32::from_rgb(220, 45, 45),
        }
    }
}

impl<'a> GraphHighlight<'a> {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn nodes(mut self, nodes: &'a [usize]) -> Self {
        self.nodes = nodes;
        self
    }
    pub fn edges(mut self, edges: &'a [usize]) -> Self {
        self.edges = edges;
        self
    }
    pub fn color(mut self, color: Color32) -> Self {
        self.color = color;
        self
    }
}

/// 2D network graph / digraph / `gplot` / tree layer (`EX-018`..`EX-022`).
#[derive(Clone, Copy, Debug)]
pub struct GraphPlot<'a> {
    pub data: &'a GraphData,
    pub name: &'a str,
    pub id_salt: Option<IdSalt>,
    pub y_axis: YAxis,
    pub node_color: Option<Color32>,
    pub node_colors: Option<&'a [Color32]>,
    pub node_scalars: Option<(&'a [f64], ColorScale)>,
    pub node_radii: Option<&'a [f32]>,
    pub node_size: f32,
    pub edge_color: Option<Color32>,
    pub edge_scalars: Option<(&'a [f64], ColorScale)>,
    pub edge_width: f32,
    pub arrow_size: f32,
    pub show_node_labels: bool,
    pub show_edge_labels: bool,
    pub highlight: Option<GraphHighlight<'a>>,
    pub style: SeriesStyle,
}

impl<'a> GraphPlot<'a> {
    pub fn new(data: &'a GraphData) -> Self {
        Self {
            data,
            name: "",
            id_salt: None,
            y_axis: YAxis::Left,
            node_color: None,
            node_colors: None,
            node_scalars: None,
            node_radii: None,
            node_size: 6.0,
            edge_color: None,
            edge_scalars: None,
            edge_width: 1.5,
            arrow_size: 9.0,
            show_node_labels: false,
            show_edge_labels: false,
            highlight: None,
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
    pub fn y_axis(mut self, axis: YAxis) -> Self {
        self.y_axis = axis;
        self
    }
    pub fn node_color(mut self, color: Color32) -> Self {
        self.node_color = Some(color);
        self
    }
    pub fn node_colors(mut self, colors: &'a [Color32]) -> Self {
        self.node_colors = Some(colors);
        self
    }
    pub fn node_scalars(mut self, scalars: &'a [f64], scale: ColorScale) -> Self {
        self.node_scalars = Some((scalars, scale));
        self
    }
    pub fn node_radii(mut self, radii: &'a [f32]) -> Self {
        self.node_radii = Some(radii);
        self
    }
    pub fn node_size(mut self, radius: f32) -> Self {
        if radius.is_finite() && radius > 0.0 {
            self.node_size = radius.clamp(1.0, 32.0);
        }
        self
    }
    pub fn edge_color(mut self, color: Color32) -> Self {
        self.edge_color = Some(color);
        self
    }
    pub fn edge_scalars(mut self, scalars: &'a [f64], scale: ColorScale) -> Self {
        self.edge_scalars = Some((scalars, scale));
        self
    }
    pub fn edge_width(mut self, width: f32) -> Self {
        if width.is_finite() && width > 0.0 {
            self.edge_width = width.clamp(0.25, 16.0);
        }
        self
    }
    pub fn arrow_size(mut self, size: f32) -> Self {
        if size.is_finite() && size > 0.0 {
            self.arrow_size = size.clamp(2.0, 28.0);
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
    pub fn highlight(mut self, highlight: GraphHighlight<'a>) -> Self {
        self.highlight = Some(highlight);
        self
    }

    pub(crate) fn resolved_node_radius(&self, node: usize) -> f32 {
        self.node_radii
            .and_then(|r| r.get(node).copied())
            .filter(|r| r.is_finite() && *r > 0.0)
            .unwrap_or(self.node_size)
    }
}
