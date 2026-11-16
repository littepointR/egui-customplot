use std::{
    collections::{HashMap, VecDeque},
    sync::{Arc, Mutex, MutexGuard, PoisonError, atomic::AtomicUsize},
};

use egui_wgpu::wgpu;

use super::{
    WgpuPlotConfig, scatter_pipeline::ScatterGpuState, waveform_pipeline::WaveformGpuState,
};

pub(super) const MAX_WAVE_COLUMNS: usize = 4_096;
pub(super) const MAX_DRAW_SLOTS: usize = 64;

pub(super) fn lock_versions(m: &Mutex<HashMap<u64, u64>>) -> MutexGuard<'_, HashMap<u64, u64>> {
    m.lock().unwrap_or_else(PoisonError::into_inner)
}

#[derive(Clone)]
pub(super) struct WgpuHostTag {
    pub max_binding_bytes: usize,
    pub max_buffer_bytes: usize,
    pub budget_bytes: Arc<AtomicUsize>,
    pub committed_versions: Arc<Mutex<HashMap<u64, u64>>>,
}

pub(super) struct CachedBuffer {
    pub buffer: wgpu::Buffer,
    pub byte_len: usize,
    pub version: u64,
}

pub(super) struct WgpuPlotResources {
    pub config: WgpuPlotConfig,
    pub sample_buffers: HashMap<u64, CachedBuffer>,
    pub lru_order: VecDeque<u64>,
    pub retained_bytes: usize,
    pub committed_versions: Arc<Mutex<HashMap<u64, u64>>>,
    pub waveform: WaveformGpuState,
    pub scatter: ScatterGpuState,
}

impl WgpuPlotResources {
    pub fn new(
        device: &wgpu::Device,
        config: WgpuPlotConfig,
        committed_versions: Arc<Mutex<HashMap<u64, u64>>>,
    ) -> Self {
        Self {
            config,
            sample_buffers: HashMap::new(),
            lru_order: VecDeque::new(),
            retained_bytes: 0,
            committed_versions,
            waveform: WaveformGpuState::new(device, config),
            scatter: ScatterGpuState::new(device, config),
        }
    }

    pub fn set_budget(&mut self, budget_bytes: usize) {
        self.config.budget_bytes = budget_bytes;
        while self.retained_bytes > self.config.budget_bytes {
            if !self.evict_one() {
                break;
            }
        }
    }

    pub fn touch_lru(&mut self, source_id: u64) {
        if let Some(pos) = self.lru_order.iter().position(|&id| id == source_id) {
            self.lru_order.remove(pos);
        }
        self.lru_order.push_front(source_id);
    }

    #[allow(clippy::too_many_arguments)]
    pub fn ensure_sample_buffer(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        source_id: u64,
        required_bytes: usize,
        usage: wgpu::BufferUsages,
        new_version: u64,
        full_data: Option<&[u8]>,
        partial_write: Option<(u64, &[u8])>,
    ) -> bool {
        if required_bytes == 0 || required_bytes > self.config.budget_bytes {
            return false;
        }
        let needs_alloc = self
            .sample_buffers
            .get(&source_id)
            .is_none_or(|b| b.byte_len < required_bytes);
        if needs_alloc {
            if let Some(old) = self.sample_buffers.remove(&source_id) {
                self.retained_bytes = self.retained_bytes.saturating_sub(old.byte_len);
                lock_versions(&self.committed_versions).remove(&source_id);
            }
            while self.retained_bytes.saturating_add(required_bytes) > self.config.budget_bytes {
                if !self.evict_one() {
                    return false;
                }
            }
            let Some(bytes) = full_data else {
                return false;
            };
            let padded = required_bytes.max(16).next_multiple_of(4);
            let buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("customplot_samples"),
                size: padded as u64,
                usage: usage | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            queue.write_buffer(&buffer, 0, bytes);
            self.retained_bytes += padded;
            self.sample_buffers.insert(
                source_id,
                CachedBuffer {
                    buffer,
                    byte_len: padded,
                    version: new_version,
                },
            );
            lock_versions(&self.committed_versions).insert(source_id, new_version);
            self.touch_lru(source_id);
            return true;
        }

        if let Some(entry) = self.sample_buffers.get_mut(&source_id) {
            if entry.version != new_version {
                if let Some((offset_bytes, slice)) = partial_write {
                    queue.write_buffer(&entry.buffer, offset_bytes, slice);
                    entry.version = new_version;
                    lock_versions(&self.committed_versions).insert(source_id, new_version);
                } else if let Some(bytes) = full_data {
                    queue.write_buffer(&entry.buffer, 0, bytes);
                    entry.version = new_version;
                    lock_versions(&self.committed_versions).insert(source_id, new_version);
                } else {
                    return false;
                }
            }
            self.touch_lru(source_id);
            return true;
        }
        false
    }

    fn evict_one(&mut self) -> bool {
        let Some(id) = self.lru_order.pop_back() else {
            return false;
        };
        if let Some(entry) = self.sample_buffers.remove(&id) {
            self.retained_bytes = self.retained_bytes.saturating_sub(entry.byte_len);
            lock_versions(&self.committed_versions).remove(&id);
        }
        true
    }
}
