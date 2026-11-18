use std::sync::Arc;

use egui::Rect;
use egui_wgpu::{CallbackResources, CallbackTrait, ScreenDescriptor, wgpu};

use super::{
    resources::{MAX_DRAW_SLOTS, MAX_WAVE_COLUMNS, SampleSource, WgpuPlotResources},
    waveform_pipeline::{WaveDrawSlot, WaveUniforms},
};

pub(super) enum WaveUpload {
    None,
    Partial {
        offset_samples: usize,
        data: Arc<[f32]>,
    },
    Full(Arc<[f32]>),
}

pub(super) struct WaveformCallback {
    pub(super) draw_key: u64,
    pub(super) source_id: SampleSource,
    pub(super) samples_version: u64,
    pub(super) total_bytes: usize,
    pub(super) upload: WaveUpload,
    pub(super) rect: Rect,
    pub(super) stroke_width: f32,
    pub(super) color: [f32; 4],
    pub(super) x_map: [f32; 4],
    pub(super) y_map: [f32; 4],
    pub(super) start_idx: u32,
    pub(super) end_idx: u32,
    pub(super) columns: u32,
    pub(super) flags: u32,
}

impl CallbackTrait for WaveformCallback {
    fn prepare(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        screen: &ScreenDescriptor,
        encoder: &mut wgpu::CommandEncoder,
        callback_resources: &mut CallbackResources,
    ) -> Vec<wgpu::CommandBuffer> {
        let Some(res) = callback_resources.get_mut::<WgpuPlotResources>() else {
            return Vec::new();
        };
        let (full_bytes, partial_bytes) = match &self.upload {
            WaveUpload::None => (None, None),
            WaveUpload::Full(arc) => (Some(bytemuck::cast_slice(arc.as_ref())), None),
            WaveUpload::Partial {
                offset_samples,
                data,
            } => (
                None,
                Some((
                    *offset_samples as u64 * 4,
                    bytemuck::cast_slice(data.as_ref()),
                )),
            ),
        };
        let ok = res.ensure_sample_buffer(
            device,
            queue,
            self.source_id,
            self.total_bytes,
            wgpu::BufferUsages::STORAGE,
            self.samples_version,
            full_bytes,
            partial_bytes,
        );
        if !ok {
            return Vec::new();
        }
        let Some(sample_buf) = res.sample_buffers.get(&self.source_id) else {
            return Vec::new();
        };
        let state = &mut res.waveform;
        if !state.slots.contains_key(&self.draw_key) {
            while state.slots.len() >= MAX_DRAW_SLOTS {
                if let Some(old) = state.slot_lru.pop_back() {
                    state.slots.remove(&old);
                } else {
                    break;
                }
            }
            let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("customplot_wave_uniforms"),
                size: size_of::<WaveUniforms>() as u64,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            let reduced_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("customplot_wave_reduced"),
                size: (MAX_WAVE_COLUMNS * 4 * 8) as u64,
                usage: wgpu::BufferUsages::STORAGE,
                mapped_at_creation: false,
            });
            let render_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("customplot_wave_render_bg"),
                layout: &state.render_bgl,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: uniform_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: reduced_buffer.as_entire_binding(),
                    },
                ],
            });
            state.slots.insert(
                self.draw_key,
                WaveDrawSlot {
                    uniform_buffer,
                    reduced_buffer,
                    compute_bind_group: None,
                    render_bind_group,
                    segment_count: 0,
                    columns: 0,
                },
            );
        }
        if let Some(pos) = state.slot_lru.iter().position(|&k| k == self.draw_key) {
            state.slot_lru.remove(pos);
        }
        state.slot_lru.push_front(self.draw_key);

        let slot = state.slots.get_mut(&self.draw_key).expect("slot exists");
        if slot
            .compute_bind_group
            .as_ref()
            .is_none_or(|(source, _)| *source != self.source_id)
        {
            let bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("customplot_wave_compute_bg"),
                layout: &state.compute_bgl,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: slot.uniform_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: sample_buf.buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: slot.reduced_buffer.as_entire_binding(),
                    },
                ],
            });
            slot.compute_bind_group = Some((self.source_id, bg));
        }

        let screen_w_pts = screen.size_in_pixels[0] as f32 / screen.pixels_per_point.max(0.01);
        let screen_h_pts = screen.size_in_pixels[1] as f32 / screen.pixels_per_point.max(0.01);
        let uniforms = WaveUniforms {
            rect: [
                self.rect.left(),
                self.rect.top(),
                self.rect.width(),
                self.rect.height(),
            ],
            screen: [
                screen_w_pts,
                screen_h_pts,
                self.stroke_width,
                screen.pixels_per_point,
            ],
            x_map: self.x_map,
            y_map: self.y_map,
            color: self.color,
            counts: [self.start_idx, self.end_idx, self.columns, self.flags],
        };
        queue.write_buffer(&slot.uniform_buffer, 0, bytemuck::bytes_of(&uniforms));
        slot.columns = self.columns;
        slot.segment_count = self.columns.saturating_mul(4).saturating_sub(1);

        let (_, bg) = slot.compute_bind_group.as_ref().expect("compute bg");
        let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("customplot_wave_reduce"),
            timestamp_writes: None,
        });
        cpass.set_pipeline(&state.compute_pipeline);
        cpass.set_bind_group(0, bg, &[]);
        cpass.dispatch_workgroups(self.columns, 1, 1);
        Vec::new()
    }

    fn paint(
        &self,
        info: egui::PaintCallbackInfo,
        render_pass: &mut wgpu::RenderPass<'static>,
        callback_resources: &CallbackResources,
    ) {
        let Some(res) = callback_resources.get::<WgpuPlotResources>() else {
            return;
        };
        let Some(slot) = res.waveform.slots.get(&self.draw_key) else {
            return;
        };
        if slot.segment_count == 0 {
            return;
        }
        render_pass.set_viewport(
            0.0,
            0.0,
            info.screen_size_px[0] as f32,
            info.screen_size_px[1] as f32,
            0.0,
            1.0,
        );
        render_pass.set_pipeline(&res.waveform.render_pipeline);
        render_pass.set_bind_group(0, &slot.render_bind_group, &[]);
        render_pass.draw(0..slot.segment_count * 6, 0..1);
    }
}
