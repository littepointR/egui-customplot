use crate::{Axis, PlotBounds, PlotTransform};

/// The vertical coordinate system of a series. Both sides share the X axis.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum YAxis {
    #[default]
    Left,
    Right,
}

#[derive(Clone, Copy)]
pub(crate) struct SeriesTransforms {
    pub left: PlotTransform,
    pub right: Option<PlotTransform>,
}

impl SeriesTransforms {
    pub fn for_axis(self, axis: YAxis) -> PlotTransform {
        if axis == YAxis::Right {
            self.right.unwrap_or(self.left)
        } else {
            self.left
        }
    }

    pub fn effective_axis(self, axis: YAxis) -> YAxis {
        if self.right.is_some() {
            axis
        } else {
            YAxis::Left
        }
    }
}

pub(crate) fn secondary_auto(
    primary: PlotTransform,
    automatic: PlotBounds,
    axis: &Axis,
) -> PlotTransform {
    let bounds = PlotBounds::new(
        [primary.bounds().min()[0], automatic.min()[1]],
        [primary.bounds().max()[0], automatic.max()[1]],
    )
    .expect("valid shared X and independent Y");
    PlotTransform::with_axes(
        bounds,
        primary.rect(),
        [primary.scales()[0], axis.scale],
        [primary.reversed()[0], axis.reversed],
    )
    .expect("valid secondary axes and layout")
}
