use std::mem::size_of;

use crate::{PlotData, PlotPoint};

#[cfg(test)]
#[path = "stream_tests.rs"]
mod tests;

/// A fixed-capacity chronological ring owned by the host, not by PlotState.
/// New samples overwrite the oldest; an oversized batch retains its last N.
/// Snapshots copy O(N) samples explicitly and are not included in this budget.
pub struct PlotStream {
    points: Box<[PlotPoint]>,
    start: usize,
    len: usize,
    revision: u64,
    last_batch: Option<u64>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StreamError {
    ZeroCapacity,
    BudgetExceeded {
        required_bytes: usize,
        budget_bytes: usize,
    },
    SizeOverflow,
    AllocationFailed,
    StaleBatch {
        batch_id: u64,
        last_batch_id: u64,
    },
    RevisionExhausted,
}

impl std::fmt::Display for StreamError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "plot stream: {self:?}")
    }
}

impl std::error::Error for StreamError {}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AppendReport {
    /// False when the last batch ID is repeated (contents are not compared).
    pub applied: bool,
    pub revision: u64,
    pub retained_input_points: usize,
    pub evicted_points: usize,
    /// Prefix of an oversized incoming batch that could not be retained.
    pub dropped_input_points: usize,
}

impl PlotStream {
    /// Budget counts fixed sample capacity only, not metadata/allocator or
    /// separately published snapshots/indexes. Allocation happens only here.
    pub fn new(capacity_points: usize, budget_bytes: usize) -> Result<Self, StreamError> {
        if capacity_points == 0 {
            return Err(StreamError::ZeroCapacity);
        }
        let required_bytes = capacity_points
            .checked_mul(size_of::<PlotPoint>())
            .ok_or(StreamError::SizeOverflow)?;
        check_budget(required_bytes, budget_bytes)?;
        let mut points = Vec::new();
        points
            .try_reserve_exact(capacity_points)
            .map_err(|_| StreamError::AllocationFailed)?;
        check_budget(points.capacity() * size_of::<PlotPoint>(), budget_bytes)?;
        points.resize(capacity_points, [0.0; 2]);
        Ok(Self {
            points: points.into_boxed_slice(),
            start: 0,
            len: 0,
            revision: 0,
            last_batch: None,
        })
    }

    pub fn len(&self) -> usize {
        self.len
    }
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
    pub fn capacity(&self) -> usize {
        self.points.len()
    }
    pub fn allocated_bytes(&self) -> usize {
        std::mem::size_of_val(self.points.as_ref())
    }
    pub fn revision(&self) -> u64 {
        self.revision
    }

    /// Apply strictly increasing host batch IDs. Repeating the last ID is a
    /// no-op, permitting egui layout retries; older IDs fail without mutation.
    /// Empty batches consume an ID but do not change the sample revision.
    /// This is overwrite-oldest, not a producer queue or lossless backpressure.
    pub fn append_once(
        &mut self,
        batch_id: u64,
        points: &[PlotPoint],
    ) -> Result<AppendReport, StreamError> {
        let mut report = AppendReport {
            applied: false,
            revision: self.revision,
            retained_input_points: 0,
            evicted_points: 0,
            dropped_input_points: 0,
        };
        if let Some(last_batch_id) = self.last_batch {
            if batch_id == last_batch_id {
                return Ok(report);
            }
            if batch_id < last_batch_id {
                return Err(StreamError::StaleBatch {
                    batch_id,
                    last_batch_id,
                });
            }
        }
        if !points.is_empty() {
            let revision = self
                .revision
                .checked_add(1)
                .ok_or(StreamError::RevisionExhausted)?;
            let capacity = self.capacity();
            report.retained_input_points = points.len().min(capacity);
            report.dropped_input_points = points.len().saturating_sub(capacity);
            report.evicted_points = self
                .len
                .saturating_sub(capacity.saturating_sub(points.len()));
            if points.len() >= capacity {
                self.points
                    .copy_from_slice(&points[points.len() - capacity..]);
                self.start = 0;
                self.len = capacity;
            } else {
                for point in points {
                    let slot = (self.start + self.len) % capacity;
                    self.points[slot] = *point;
                    if self.len == capacity {
                        self.start = (self.start + 1) % capacity;
                    } else {
                        self.len += 1;
                    }
                }
            }
            self.revision = revision;
        }
        self.last_batch = Some(batch_id);
        report.applied = true;
        report.revision = self.revision;
        Ok(report)
    }

    /// Chronological slices, without copying or allocation.
    pub fn as_slices(&self) -> (&[PlotPoint], &[PlotPoint]) {
        let first_len = self.len.min(self.capacity() - self.start);
        (
            &self.points[self.start..self.start + first_len],
            &self.points[..self.len - first_len],
        )
    }

    /// Publish explicitly after batching, not inside each Plot::show pass.
    /// Old snapshots remain immutable and alive only while their host owns them.
    /// A snapshot has a new PlotData version and no prebuilt line index.
    pub fn snapshot(&self) -> PlotData {
        let (a, b) = self.as_slices();
        PlotData::new_unindexed(a.iter().chain(b).copied().collect::<Vec<_>>())
    }

    /// Keeps the batch watermark, preventing old/retried batches after a clear.
    pub fn clear(&mut self) -> Result<(), StreamError> {
        if !self.is_empty() {
            self.revision = self
                .revision
                .checked_add(1)
                .ok_or(StreamError::RevisionExhausted)?;
            self.start = 0;
            self.len = 0;
        }
        Ok(())
    }
}

fn check_budget(required_bytes: usize, budget_bytes: usize) -> Result<(), StreamError> {
    if required_bytes > budget_bytes {
        Err(StreamError::BudgetExceeded {
            required_bytes,
            budget_bytes,
        })
    } else {
        Ok(())
    }
}
