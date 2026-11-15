//! CPU geometry preparation and ordered egui Painter submission.

mod annotation;
mod axes;
mod line;
mod markers;
mod overlay;
mod pattern;
mod series;

pub(crate) use annotation::paint_annotations;
pub(crate) use axes::{AxesStyle, paint_axes};
pub(crate) use overlay::paint_hover;
pub(crate) use series::paint_series;
