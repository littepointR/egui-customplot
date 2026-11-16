use std::sync::Arc;

use egui::Rect;
use egui_wgpu::{CallbackResources, CallbackTrait, ScreenDescriptor, wgpu};

use super::{
    resources::{MAX_DRAW_SLOTS, WgpuPlotResources},
    scatter_pipeline::{GpuScatterInstance, ScatterDrawSlot, ScatterUniforms},
};

pub(super) struct ScatterCallback {
    pub(super) draw_key: u64,
    pub(super) source_id: u64,
    pub(super) version: u64,
    pub(super) total_bytes: usize,
    pub(super) upload: Option<Arc<[GpuScatterInstance]>>,
    pub(super) first_instance: u32,
    pub(super) instance_count: u32,
    pub(super) rect: Rect,
    pub(super) default_radius: f32,
    pub(super) default_color: [f32; 4],
    pub(super) axis_min: [f32; 4],
    pub(super) ref_origin: [f32; 4],
}

impl CallbackTrait for ScatterCallback {
    fn prepare(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        screen: &ScreenDescriptor,
        _encoder: &mut wgpu::CommandEncoder,
        callback_resources: &mut CallbackResources,
    ) -> Vec<wgpu::CommandBuffer> {
        let Some(res) = callback_resources.get_mut::<WgpuPlotResources>() else {
            return Vec::new();
        };
        let bytes_opt = self
            .upload
            .as_ref()
            .map(|a| bytemuck::cast_slice(a.as_ref()));
        let ok = res.ensure_sample_buffer(
            device,
            queue,
            self.source_id,
            self.total_bytes,
            wgpu::BufferUsages::VERTEX,
            self.version,
            bytes_opt,
            None,
        );
        if !ok {
            return Vec::new();
        }
        let state = &mut res.scatter;
        if !state.slots.contains_key(&self.draw_key) {
            while state.slots.len() >= MAX_DRAW_SLOTS {
                if let Some(old) = state.slot_lru.pop_back() {
                    state.slots.remove(&old);
                } else {
                    break;
                }
            }
            let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("customplot_scatter_uniforms"),
                size: size_of::<ScatterUniforms>() as u64,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("customplot_scatter_bg"),
                layout: &state.bgl,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniform_buffer.as_entire_binding(),
                }],
            });
            state.slots.insert(
                self.draw_key,
                ScatterDrawSlot {
                    uniform_buffer,
                    bind_group,
                    first_instance: 0,
                    instance_count: 0,
                },
            );
        }
        if let Some(pos) = state.slot_lru.iter().position(|&k| k == self.draw_key) {
            state.slot_lru.remove(pos);
        }
        state.slot_lru.push_front(self.draw_key);
        let slot = state.slots.get_mut(&self.draw_key).expect("slot");
        let screen_w = screen.size_in_pixels[0] as f32 / screen.pixels_per_point.max(0.01);
        let screen_h = screen.size_in_pixels[1] as f32 / screen.pixels_per_point.max(0.01);
        let uniforms = ScatterUniforms {
            rect: [
                self.rect.left(),
                self.rect.top(),
                self.rect.width(),
                self.rect.height(),
            ],
            screen: [
                screen_w,
                screen_h,
                self.default_radius,
                screen.pixels_per_point,
            ],
            axis_min: self.axis_min,
            ref_origin: self.ref_origin,
            default_color: self.default_color,
        };
        queue.write_buffer(&slot.uniform_buffer, 0, bytemuck::bytes_of(&uniforms));
        slot.first_instance = self.first_instance;
        slot.instance_count = self.instance_count;
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
        let Some(slot) = res.scatter.slots.get(&self.draw_key) else {
            return;
        };
        let Some(vbuf) = res.sample_buffers.get(&self.source_id) else {
            return;
        };
        if slot.instance_count == 0 {
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
        render_pass.set_pipeline(&res.scatter.pipeline);
        render_pass.set_bind_group(0, &slot.bind_group, &[]);
        render_pass.set_vertex_buffer(0, vbuf.buffer.slice(..self.total_bytes as u64));
        render_pass.draw(
            0..6,
            slot.first_instance..slot.first_instance.saturating_add(slot.instance_count),
        );
    }
}
