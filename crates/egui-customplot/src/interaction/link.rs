use egui::Context;

use crate::{AxisScale, PlotBounds, PlotState};

/// Host-owned axis-link group. Show every member, then call `synchronize` once
/// per UI pass. Only data ranges are shared: samples, visibility, axis direction
/// and each unlinked axis remain independent. No states are retained here.
///
/// Shared axes must use the same scale. Equal-aspect plots are rejected rather
/// than silently weakening either constraint. A state belongs to at most one
/// group per axis. Cross-Context/viewport linking is not supported.
///
/// 宿主持有的多图坐标轴联动器：在同一 `Ui` pass 内展示全部成员后调用一次 `synchronize`，
/// 仅同步勾选轴的数据视口范围，不干扰成员各自的样本、图例显隐、轴反转与未联动轴。
pub struct AxisLink {
    axes: [bool; 2],
    manual: [Option<[f64; 2]>; 2],
    was_reset: bool,
}

/// Validation failure returned by [`AxisLink::synchronize`] when linking constraints are violated.
///
/// [`AxisLink::synchronize`] 在轴缩放类型不一致、存在等比例约束或成员未在当前帧绘制时返回的错误。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LinkError {
    NotShown { member: usize },
    StaleView { member: usize },
    ScaleMismatch { member: usize, axis: usize },
    AspectConstraint { member: usize },
    InvalidRange { axis: usize },
}

impl std::fmt::Display for LinkError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "axis link: {self:?}")
    }
}
impl std::error::Error for LinkError {}

/// Arbitration summary returned by [`AxisLink::synchronize`] (shared ranges and conflict count).
///
/// [`AxisLink::synchronize`] 返回的同步汇总：最终共同范围、发生变化的成员数与首动作来源索引。
#[derive(Clone, Debug, Default, PartialEq)]
pub struct LinkReport {
    pub ranges: [Option<[f64; 2]>; 2],
    pub changed_plots: usize,
    /// First member with a navigation action per axis. Deterministic input
    /// arbitration; later conflicting actions are reported, not accumulated.
    pub sources: [Option<usize>; 2],
    pub conflicting_actions: usize,
}

impl AxisLink {
    pub fn new(axes: [bool; 2]) -> Self {
        Self {
            axes,
            manual: [None; 2],
            was_reset: false,
        }
    }

    pub fn axes(&self) -> [bool; 2] {
        self.axes
    }

    /// Set shared manual ranges on enabled axes. Domain compatibility is checked
    /// transactionally against every member at the next synchronize call.
    pub fn set_bounds(&mut self, bounds: PlotBounds) {
        self.was_reset = false;
        for axis in 0..2 {
            if self.axes[axis] {
                self.manual[axis] = Some([bounds.min()[axis], bounds.max()[axis]]);
            }
        }
    }

    /// Return shared axes to the union of member auto-fit ranges. Resetting any
    /// linked plot with Home/double-click has the same group-wide effect.
    pub fn reset(&mut self) {
        self.manual = [None; 2];
        self.was_reset = true;
    }

    /// Validate all members before mutation, then propagate ranges. Without an
    /// explicit/manual navigation range, continuously fit the union of cached
    /// automatic ranges, so changing data does not freeze an unlinked axis.
    ///
    /// Changes request a discarded layout pass plus a repaint. With egui's pass
    /// limit exhausted, earlier members settle on the next frame; this method
    /// never changes the host's pass limit or publishes once-only events.
    pub fn synchronize(
        &mut self,
        ctx: &Context,
        states: &mut [&mut PlotState],
    ) -> Result<LinkReport, LinkError> {
        let mut report = LinkReport::default();
        if states.is_empty() || !self.axes.iter().any(|a| *a) {
            return Ok(report);
        }
        let mut scales: Option<[AxisScale; 2]> = None;
        for (member, state) in states.iter().enumerate() {
            let frame = state.frame.as_ref().ok_or(LinkError::NotShown { member })?;
            if !frame.is_current(ctx) {
                return Err(LinkError::StaleView { member });
            }
            let view = frame.view.ok_or(LinkError::NotShown { member })?;
            if view.shown.data_aspect().is_some() {
                return Err(LinkError::AspectConstraint { member });
            }
            if let Some(first) = scales {
                for axis in 0..2 {
                    if self.axes[axis] && first[axis] != view.shown.scales()[axis] {
                        return Err(LinkError::ScaleMismatch { member, axis });
                    }
                }
            } else {
                scales = Some(view.shown.scales());
            }
        }
        let scales = scales.expect("nonempty members");
        let mut manual = std::array::from_fn(|axis| {
            if !self.axes[axis] || self.was_reset {
                None
            } else {
                self.manual[axis].or_else(|| {
                    states.iter().find_map(|s| {
                        s.linked_manual[axis]
                            .then_some(s.linked_ranges[axis])
                            .flatten()
                    })
                })
            }
        });
        for (member, state) in states.iter().enumerate() {
            let view = state.frame.as_ref().unwrap().view.unwrap();
            for (axis, enabled) in self.axes.into_iter().enumerate() {
                if !enabled || !(view.reset || view.changed_axes[axis]) {
                    continue;
                }
                if report.sources[axis].is_some() {
                    report.conflicting_actions += 1;
                    continue;
                }
                report.sources[axis] = Some(member);
                manual[axis] = if view.reset {
                    None
                } else {
                    Some([
                        view.shown.bounds().min()[axis],
                        view.shown.bounds().max()[axis],
                    ])
                };
            }
        }
        for axis in 0..2 {
            if !self.axes[axis] {
                continue;
            }
            let range = manual[axis].unwrap_or_else(|| {
                states
                    .iter()
                    .fold([f64::INFINITY, f64::NEG_INFINITY], |range, state| {
                        let bounds = state.frame.as_ref().unwrap().view.unwrap().auto.bounds();
                        [
                            range[0].min(bounds.min()[axis]),
                            range[1].max(bounds.max()[axis]),
                        ]
                    })
            });
            let span = range[1] - range[0];
            let transformed = [
                scales[axis].forward(range[0]),
                scales[axis].forward(range[1]),
            ];
            if !span.is_finite()
                || span <= 0.0
                || !matches!(transformed, [Some(lo), Some(hi)] if (hi - lo).is_finite() && hi > lo)
            {
                return Err(LinkError::InvalidRange { axis });
            }
            report.ranges[axis] = Some(range);
        }
        self.manual = manual;
        self.was_reset = false;
        for state in states.iter_mut() {
            let mut changed = false;
            for axis in 0..2 {
                if self.axes[axis] {
                    let is_manual = manual[axis].is_some();
                    if state.linked_ranges[axis] != report.ranges[axis] {
                        state.linked_ranges[axis] = report.ranges[axis];
                        changed = true;
                    }
                    state.linked_manual[axis] = is_manual;
                }
            }
            report.changed_plots += usize::from(changed);
        }
        if report.changed_plots > 0 {
            ctx.request_discard("synchronized plot axis ranges");
            ctx.request_repaint();
        }
        Ok(report)
    }
}
