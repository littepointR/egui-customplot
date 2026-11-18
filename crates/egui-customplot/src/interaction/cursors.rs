use crate::{PlotPoint, PlotResponse, YAxis};
use egui::{Context, Id, Pos2, Ui, ViewportId};

/// A sample (when picked) or pointer coordinate pinned on a Cartesian plot.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CursorReading {
    pub point: PlotPoint,
    pub plot: Id,
    pub axis: YAxis,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct CursorMeasurements {
    pub a: Option<CursorReading>,
    pub b: Option<CursorReading>,
}
impl CursorMeasurements {
    pub fn delta_x(self) -> Option<f64> {
        Some(self.b?.point[0] - self.a?.point[0])
    }
    /// Vertical difference is meaningful only on the same source plot and Y axis.
    pub fn delta_y(self) -> Option<f64> {
        let (a, b) = (self.a?, self.b?);
        (a.plot == b.plot && a.axis == b.axis).then_some(b.point[1] - a.point[1])
    }
}

#[derive(Clone, Debug)]
struct Pin {
    ctx: Context,
    viewport: ViewportId,
    frame: u64,
    plot: Id,
    pos: Pos2,
    b: bool,
}

/// Host-owned A/B measurement state for Cartesian plots sharing the same X units.
/// After drawing all members on one layer, call `show` once on each UI pass.
/// Click to set A, Shift-click to set B. Zooming preserves the stored coordinates.
#[derive(Clone, Debug, Default)]
pub struct SharedCursors {
    readings: CursorMeasurements,
    pending: Option<Pin>,
}
impl SharedCursors {
    pub fn clear(&mut self) {
        *self = Self::default();
    }
    pub fn measurements(&self) -> CursorMeasurements {
        self.readings
    }
    pub fn show(&mut self, ui: &Ui, plots: &[&PlotResponse]) -> CursorMeasurements {
        let ctx = ui.ctx();
        let viewport = ctx.viewport_id();
        let frame = ctx.cumulative_frame_nr();
        if self
            .pending
            .as_ref()
            .is_some_and(|p| p.ctx != *ctx || p.viewport != viewport || p.frame != frame)
        {
            self.pending = None;
        }
        let mut hover = None;
        for &r in plots {
            if r.response.layer_id != ui.layer_id() || r.response.ctx != *ctx {
                continue;
            }
            if let Some(pos) = r.response.hover_pos()
                && r.transform.rect().contains(pos)
            {
                hover = Some(read(r, pos).point[0]);
                if r.response.clicked() {
                    self.pending = Some(Pin {
                        ctx: ctx.clone(),
                        viewport,
                        frame,
                        plot: r.response.id,
                        pos,
                        b: ui.input(|i| i.modifiers.shift),
                    });
                }
            }
            if let Some(pin) = &self.pending
                && pin.plot == r.response.id
            {
                let reading = Some(read(r, pin.pos));
                if pin.b {
                    self.readings.b = reading;
                } else {
                    self.readings.a = reading;
                }
            }
        }
        for &r in plots {
            if r.response.layer_id == ui.layer_id() && r.response.ctx == *ctx {
                crate::render::paint_shared_cursors(ui, r.transform, hover, self.readings);
            }
        }
        self.readings
    }
}

fn read(r: &PlotResponse, pos: Pos2) -> CursorReading {
    let (point, axis) = r.hovered_point.as_ref().map_or_else(
        || (r.transform.point_from_position(pos), YAxis::Left),
        |p| (p.point, p.y_axis),
    );
    CursorReading {
        point,
        axis,
        plot: r.response.id,
    }
}
