//! Application-owned interaction state, input arbitration and raw-point picking.

mod bar;
mod cell;
mod hover;
mod link;
mod navigation;
mod picking;
mod point_aux;
mod quiver;
mod region;
mod secondary;
mod state;
mod tooltip;

pub use bar::HoveredBar;
pub use cell::HoveredCell;
pub(crate) use cell::pick_cell;
pub(crate) use hover::{HoverPick, pick_hover};
pub use link::{AxisLink, LinkError, LinkReport};
pub(crate) use navigation::{NavigationOptions, navigate};
pub use picking::HoveredPoint;
pub use quiver::{HoveredVector, VectorAnchor};
pub use region::{HoveredRegion, RegionKind};
pub(crate) use secondary::navigate_secondary;
pub use state::PlotState;
pub use tooltip::HoverInfo;
pub(crate) use tooltip::{HoverFormatter, show_tooltip};
