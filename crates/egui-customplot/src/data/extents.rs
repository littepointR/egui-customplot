use crate::{AxisScale, PlotPoint};

type Extent = (PlotPoint, PlotPoint);

/// Fixed-size preprocessing summary for independent data-space primitives.
/// A primitive is accepted only when its entire extent is valid in the domain.
#[derive(Clone, Debug, Default)]
pub(super) struct CachedExtents {
    extents: [Option<Extent>; 4],
    invalid: [usize; 4],
}

impl CachedExtents {
    pub fn push(&mut self, extent: Option<Extent>) {
        for domain in 0..4 {
            let scales = [
                if domain & 1 != 0 {
                    AxisScale::Log10
                } else {
                    AxisScale::Linear
                },
                if domain & 2 != 0 {
                    AxisScale::Log10
                } else {
                    AxisScale::Linear
                },
            ];
            let valid = extent.filter(|(lo, hi)| {
                (0..2).all(|i| {
                    lo[i] <= hi[i]
                        && (hi[i] - lo[i]).is_finite()
                        && scales[i].forward(lo[i]).is_some()
                        && scales[i].forward(hi[i]).is_some()
                })
            });
            if let Some((lo, hi)) = valid {
                let (min, max) = self.extents[domain].get_or_insert((lo, hi));
                for axis in 0..2 {
                    min[axis] = min[axis].min(lo[axis]);
                    max[axis] = max[axis].max(hi[axis]);
                }
            } else {
                self.invalid[domain] += 1;
            }
        }
    }
    pub fn extent(&self, scales: [AxisScale; 2]) -> Option<Extent> {
        self.extents[domain(scales)]
    }
    pub fn invalid(&self, scales: [AxisScale; 2]) -> usize {
        self.invalid[domain(scales)]
    }
}

fn domain(scales: [AxisScale; 2]) -> usize {
    usize::from(scales[0] == AxisScale::Log10) + usize::from(scales[1] == AxisScale::Log10) * 2
}
