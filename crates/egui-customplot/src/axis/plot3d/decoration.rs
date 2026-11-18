//! Placement and drawing hooks for the annotations of one Cartesian 3D axis.
use crate::Transform3D;
use egui::{Align2, Color32, FontId, Painter, Pos2, Vec2};

/// Selects min/max **data** ends of the other two axes, in X/Y/Z order.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AxisEdge3D {
    pub first_max: bool,
    pub second_max: bool,
}

impl AxisEdge3D {
    pub const fn new(first_max: bool, second_max: bool) -> Self {
        Self {
            first_max,
            second_max,
        }
    }
    pub(crate) fn unit(self, axis: usize, reversed: [bool; 3], along: f64) -> [f64; 3] {
        let mut ends = [self.first_max, self.second_max].into_iter();
        std::array::from_fn(|i| {
            if i == axis {
                along
            } else {
                f64::from(ends.next().unwrap() ^ reversed[i])
            }
        })
    }
}

#[derive(Clone, Debug)]
pub struct AxisLabel3D {
    /// Empty for an axis title, the data value for a tick.
    pub value: Option<f64>,
    pub text: String,
    pub position: Pos2,
    pub align: Align2,
    pub font: FontId,
    pub color: Color32,
}

/// All geometry is expressed in logical egui points. Called during layout retries and export.
#[derive(Clone, Copy, Debug)]
pub struct AxisLayoutContext3D {
    /// X = 0, Y = 1, Z = 2.
    pub axis: usize,
    pub transform: Transform3D,
    pub previous: Option<AxisEdge3D>,
}

/// Override only the operations that differ from the default annotation layout.
pub trait AxisDecoration3D: std::fmt::Debug + Send + Sync {
    fn select_edge(&self, _context: &AxisLayoutContext3D, suggested: AxisEdge3D) -> AxisEdge3D {
        suggested
    }
    fn layout_labels(
        &self,
        _context: &AxisLayoutContext3D,
        _edge: AxisEdge3D,
        _labels: &mut Vec<AxisLabel3D>,
    ) {
    }
    fn paint_label(&self, painter: &Painter, label: &AxisLabel3D) {
        label.paint(painter);
    }
}

impl AxisLabel3D {
    pub fn paint(&self, painter: &Painter) {
        painter.text(
            self.position,
            self.align,
            &self.text,
            self.font.clone(),
            self.color,
        );
    }
}

pub(crate) fn edge_geometry(
    context: AxisLayoutContext3D,
    edge: AxisEdge3D,
) -> Option<(Pos2, Vec2, f32)> {
    let t = context.transform;
    let at = |u| {
        t.project_unit(edge.unit(context.axis, t.reversed(), u))
            .map(|p| p.0)
    };
    let (a, b) = (at(0.)?, at(1.)?);
    let mid = at(0.5)?;
    let center = t.project_unit([0.5; 3])?.0;
    if a.distance_sq(b) < 1.0 {
        return None;
    }
    let mut normal = (b - a).normalized().rot90();
    if normal.dot(mid - center) < 0. {
        normal = -normal;
    }
    Some((mid, normal, (mid - center).dot(normal)))
}
