use super::*;
use crate::{Axis, AxisScale, Plot, PlotState, Scatter, UniformWaveform};

fn input() -> egui::RawInput {
    egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::vec2(800.0, 900.0),
        )),
        ..Default::default()
    }
}

fn install_host_tag(ctx: &egui::Context) -> WgpuHostTag {
    let tag = WgpuHostTag {
        max_binding_bytes: 1 << 20,
        max_buffer_bytes: 1 << 20,
        budget_bytes: Arc::new(AtomicUsize::new(1 << 20)),
        committed_versions: Default::default(),
    };
    ctx.data_mut(|d| d.insert_temp(egui::Id::NULL, tag.clone()));
    tag
}

#[test]
fn registered_gpu_preserves_exact_waveform_sampling() {
    let ctx = egui::Context::default();
    install_host_tag(&ctx);
    let samples: Vec<_> = (0..2048).map(|i| (i % 5) as f32).collect();
    let wave = UniformWaveform::from_f32(0.0, 1.0, samples, 1 << 20).unwrap();
    let mut state = PlotState::default();
    for sampling in [
        LineSampling::Exact,
        LineSampling::Visible,
        LineSampling::MinMax,
    ] {
        ctx.run_ui(input(), |ui| {
            let line = wave.line().backend(RenderBackend::Wgpu).sampling(sampling);
            let response = Plot::new("wave").show_waveform(ui, &mut state, &[line]);
            let diag = response.diagnostics;
            if sampling == LineSampling::MinMax {
                assert_eq!(diag.wgpu_series, 1);
            } else {
                assert_eq!(diag.wgpu_series, 0);
                assert_eq!(diag.wgpu_fallback_series, 1);
                assert!(diag.output_points >= wave.len());
            }
        })
        .drop_without_applying_deltas();
    }
}

#[test]
fn scatter_upload_is_reused_only_for_matching_axis_encodings() {
    let ctx = egui::Context::default();
    let tag = install_host_tag(&ctx);
    let data = PlotData::new(vec![[10.0, 25.0], [25.0, 50.0]]);
    resources::lock_versions(&tag.committed_versions).insert(
        resources::SampleSource::Scatter {
            version: data.version(),
            log_axes: [false; 2],
        },
        data.version(),
    );
    for x in [AxisScale::Linear, AxisScale::Log10] {
        for y in [AxisScale::Linear, AxisScale::Log10] {
            ctx.run_ui(input(), |ui| {
                let scatter = Scatter::new(&data).backend(RenderBackend::Wgpu);
                let response = Plot::new("scatter")
                    .x_axis(Axis::new().scale(x))
                    .y_axis(Axis::new().scale(y))
                    .show_series(ui, &mut PlotState::default(), &[scatter.into()]);
                assert_eq!(response.diagnostics.wgpu_series, 1);
                assert_eq!(
                    response.diagnostics.wgpu_uploaded_bytes == 0,
                    [x, y] == [AxisScale::Linear; 2]
                );
            })
            .drop_without_applying_deltas();
        }
    }
}

#[test]
#[ignore = "requires a native compute-capable wgpu adapter"]
fn native_gpu_isolates_plot_draws_scale_encodings_and_eviction() {
    let ctx = egui::Context::default();
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
    let config = egui_wgpu::WgpuConfiguration::default();
    let host = pollster::block_on(RenderState::create(
        &config,
        &instance,
        None,
        Default::default(),
    ))
    .expect("native GPU adapter");
    eprintln!(
        "GPU callback regression adapter: {:?}",
        host.adapter.get_info()
    );
    assert!(register_wgpu(&ctx, &host));

    let wave = UniformWaveform::from_f32(0.0, 1.0, vec![1.0, 2.0, 1.0, 3.0], 1 << 20).unwrap();
    let scatter = PlotData::new(vec![[1.0, -1.0], [2.0, 1.0], [3.0, 4.0]]);
    let mut states: [PlotState; 4] = Default::default();
    let mut expected_pixels = Vec::new();
    let mut output = ctx.run_ui(input(), |ui| {
        expected_pixels.clear();
        for (idx, state) in states[..2].iter_mut().enumerate() {
            let line = wave
                .line()
                .backend(RenderBackend::Wgpu)
                .width(5.0)
                .color(Color32::RED);
            let r = Plot::new(("wave", idx))
                .height(180.0)
                .show_waveform(ui, state, &[line]);
            assert_eq!(r.diagnostics.wgpu_series, 1);
            expected_pixels.push(r.transform.position_from_point([1.5, 1.5]));
        }
        for (state, scale) in states[2..]
            .iter_mut()
            .zip([AxisScale::Linear, AxisScale::Log10])
        {
            let points = Scatter::new(&scatter)
                .backend(RenderBackend::Wgpu)
                .color(Color32::RED)
                .radius(4.0);
            let r = Plot::new(("scatter", scale == AxisScale::Log10))
                .height(180.0)
                .y_axis(Axis::new().scale(scale))
                .show_series(ui, state, &[points.into()]);
            assert_eq!(r.diagnostics.wgpu_series, 1);
            expected_pixels.push(r.transform.position_from_point([2.0, 1.0]));
        }
    });
    let mut jobs = ctx.tessellate(output.shapes, output.pixels_per_point);
    jobs.retain(|job| matches!(job.primitive, egui::epaint::Primitive::Callback(_)));
    output.textures_delta.clear();
    let screen = egui_wgpu::ScreenDescriptor {
        size_in_pixels: [800, 900],
        pixels_per_point: output.pixels_per_point,
    };
    let mut encoder = host.device.create_command_encoder(&Default::default());
    let mut renderer = host.renderer.write();
    let prepare = renderer.update_buffers(&host.device, &host.queue, &mut encoder, &jobs, &screen);
    let target = host.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("GPU plot regression target"),
        size: wgpu::Extent3d {
            width: 800,
            height: 900,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: host.target_format,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let view = target.create_view(&Default::default());
    let pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            view: &view,
            depth_slice: None,
            resolve_target: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                store: wgpu::StoreOp::Store,
            },
        })],
        ..Default::default()
    });
    renderer.render(&mut pass.forget_lifetime(), &jobs, &screen);
    let pixels = host.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("plot pixel readback"),
        size: (expected_pixels.len() * 4) as u64,
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    for (i, pos) in expected_pixels.iter().enumerate() {
        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                origin: wgpu::Origin3d {
                    x: pos.x as u32,
                    y: pos.y as u32,
                    z: 0,
                },
                ..target.as_image_copy()
            },
            wgpu::TexelCopyBufferInfo {
                buffer: &pixels,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: (4 * i) as u64,
                    ..Default::default()
                },
            },
            wgpu::Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            },
        );
    }
    host.queue
        .submit(prepare.into_iter().chain([encoder.finish()]));
    let (tx, rx) = std::sync::mpsc::channel();
    pixels
        .slice(..)
        .map_async(wgpu::MapMode::Read, move |result| tx.send(result).unwrap());
    host.device
        .poll(wgpu::PollType::wait_indefinitely())
        .unwrap();
    rx.recv().unwrap().unwrap();
    for pixel in pixels
        .slice(..)
        .get_mapped_range()
        .unwrap()
        .as_chunks::<4>()
        .0
    {
        assert_eq!(pixel, &[255, 0, 0, 255]);
    }
    let resources = renderer
        .callback_resources
        .get_mut::<WgpuPlotResources>()
        .unwrap();
    assert_eq!(resources.waveform.slots.len(), 2);
    assert_eq!(resources.scatter.slots.len(), 2);
    assert_eq!(resources.sample_buffers.len(), 3);
    assert!(
        resources
            .waveform
            .slots
            .values()
            .all(|s| s.compute_bind_group.is_some())
    );
    resources.set_budget(0);
    assert!(resources.sample_buffers.is_empty());
    assert!(
        resources
            .waveform
            .slots
            .values()
            .all(|s| s.compute_bind_group.is_none())
    );
}
