//! Coordinate domains, validated bounds, transforms and tick configuration.

mod aspect;
mod bounds;
mod scale;
mod ticks;
mod transform;

pub use bounds::{BoundsError, PlotBounds};
pub use scale::AxisScale;
pub use ticks::{Axis, AxisTick};
pub use transform::PlotTransform;
