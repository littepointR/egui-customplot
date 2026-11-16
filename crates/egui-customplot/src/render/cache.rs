use std::{collections::VecDeque, mem::size_of};

use super::line::PreparedLine;
use crate::{AxisScale, Line, LineInterpolation, LineSampling, PlotTransform};

const MAX_ENTRIES: usize = 64;

/// Host-owned LRU of clipped, unstyled line coordinates. Never retains samples
/// or PlotState. Opt in with `Plot::show_cached` / `show_series_cached`.
///
/// The budget counts occupied entry metadata and retained Vec capacities, not
/// allocator headers, VecDeque spare slots, temporary clones submitted to egui,
/// markers, tessellation or GPU resources. At most 64 entries are retained.
/// A hit still copies coordinates for egui and runs dash/tessellation work.
#[derive(Clone, Debug, Default)]
pub struct PlotCache {
    entries: VecDeque<Entry>,
    budget: usize,
    bytes: usize,
}

#[derive(Clone, Debug)]
struct Entry {
    key: Key,
    geometry: PreparedLine,
    bytes: usize,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct Key {
    source_tag: u8,
    version: u64,
    bounds: [u64; 4],
    rect: [u32; 4],
    dpi: u32,
    scales: [AxisScale; 2],
    reversed: [bool; 2],
    sampling: LineSampling,
    interpolation: LineInterpolation,
    indexed: bool,
}

impl Key {
    pub fn new(line: Line<'_>, transform: PlotTransform, dpi: f32) -> Self {
        Self::from_parts(
            0,
            line.data.version(),
            transform,
            dpi,
            line.sampling,
            line.interpolation,
            line.data.line_index().is_some(),
        )
    }

    pub fn from_parts(
        source_tag: u8,
        version: u64,
        transform: PlotTransform,
        dpi: f32,
        sampling: LineSampling,
        interpolation: LineInterpolation,
        indexed: bool,
    ) -> Self {
        let bounds = transform.bounds();
        let rect = transform.rect();
        Self {
            source_tag,
            version,
            bounds: [
                bounds.min()[0],
                bounds.min()[1],
                bounds.max()[0],
                bounds.max()[1],
            ]
            .map(f64::to_bits),
            rect: [rect.left(), rect.top(), rect.right(), rect.bottom()].map(f32::to_bits),
            dpi: dpi.to_bits(),
            scales: transform.scales(),
            reversed: transform.reversed(),
            sampling,
            interpolation,
            indexed,
        }
    }
}

impl PlotCache {
    /// Zero (also Default) disables retention, but still permits normal drawing.
    pub fn new(budget_bytes: usize) -> Self {
        Self {
            budget: budget_bytes,
            ..Self::default()
        }
    }

    pub fn budget_bytes(&self) -> usize {
        self.budget
    }
    pub fn retained_bytes(&self) -> usize {
        self.bytes
    }
    pub fn entry_count(&self) -> usize {
        self.entries.len()
    }

    /// Immediately evicts least-recently-used entries to meet the new budget.
    pub fn set_budget(&mut self, budget_bytes: usize) {
        self.budget = budget_bytes;
        while self.bytes > self.budget {
            self.evict();
        }
        if self.entries.is_empty() {
            self.clear();
        }
    }

    /// Releases retained geometry and container capacity.
    pub fn clear(&mut self) {
        self.entries = VecDeque::new();
        self.bytes = 0;
    }

    pub(super) fn get(&mut self, key: &Key) -> Option<PreparedLine> {
        let position = self.entries.iter().position(|e| e.key == *key)?;
        let entry = self.entries.remove(position).expect("known entry");
        let result = entry.geometry.clone();
        self.entries.push_front(entry);
        Some(result)
    }

    /// Oversized results are drawn but not retained. No eviction on rejection.
    pub(super) fn insert(&mut self, key: Key, geometry: &PreparedLine) -> bool {
        // Vec::clone has its own (usually exact) capacities: count the actual
        // retained clone, not the larger source scratch allocation.
        let minimum = size_of::<Entry>()
            + geometry.paths.len() * size_of::<Vec<egui::Pos2>>()
            + geometry
                .paths
                .iter()
                .map(|p| p.len() * size_of::<egui::Pos2>())
                .sum::<usize>();
        if minimum > self.budget {
            return false;
        }
        let geometry = geometry.clone();
        let bytes = size_of::<Entry>()
            + geometry.paths.capacity() * size_of::<Vec<egui::Pos2>>()
            + geometry
                .paths
                .iter()
                .map(|p| p.capacity() * size_of::<egui::Pos2>())
                .sum::<usize>();
        if bytes > self.budget {
            return false;
        }
        while self.bytes > self.budget - bytes || self.entries.len() >= MAX_ENTRIES {
            self.evict();
        }
        self.bytes += bytes;
        self.entries.push_front(Entry {
            key,
            geometry,
            bytes,
        });
        true
    }

    fn evict(&mut self) {
        if let Some(entry) = self.entries.pop_back() {
            self.bytes -= entry.bytes;
        }
    }
}
