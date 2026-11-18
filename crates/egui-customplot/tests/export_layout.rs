#[path = "support/input.rs"]
mod support;

use egui::Context;
use egui_customplot::{LegendPosition, Line, Plot, PlotData, PlotState};

#[test]
fn outside_legends_reserve_space_and_report_their_hit_targets() {
    let data = PlotData::new(vec![[0.0, 0.0], [1.0, 1.0]]);
    for position in [LegendPosition::OutsideRight, LegendPosition::OutsideBottom] {
        let mut geometry = None;
        Context::default()
            .run_ui(support::input(vec![]), |ui| {
                let response = Plot::new("external").legend_position(position).show(
                    ui,
                    &mut PlotState::default(),
                    &[Line::new(&data).name("measured temperature")],
                );
                geometry = Some((
                    response.outer_rect,
                    response.transform.rect(),
                    response.legend_entries,
                ));
            })
            .drop_without_applying_deltas();
        let (outer, plot, entries) = geometry.unwrap();
        assert_eq!(entries.len(), 1);
        let hit = entries[0].rect;
        assert!(hit.is_positive() && outer.contains_rect(hit));
        match position {
            LegendPosition::OutsideRight => assert!(hit.left() > plot.right()),
            LegendPosition::OutsideBottom => assert!(hit.top() > plot.bottom()),
            _ => unreachable!(),
        }
    }
}

#[cfg(feature = "export")]
#[test]
fn target_relayout_uses_requested_canvas_density_and_depth_resolution() {
    use egui_customplot::{DepthMode, ExportOptions, Plot3D, Plot3DState, PlotSnapshot};
    let ctx = Context::default();
    let mut sizes = Vec::new();
    for width in [480, 800] {
        let size = egui::vec2(width as f32, 400.0);
        let mut state = Plot3DState::default();
        let mut pixels = 0;
        let snap = PlotSnapshot::render_at(&ctx, size, 2.0, Default::default(), |ui| {
            let response = Plot3D::new("depth")
                .height(400.0)
                .depth_mode(DepthMode::ZBuffer)
                .show(ui, &mut state, &[]);
            pixels = response.diagnostics.plot3d.zbuffer_pixels;
            egui::Rect::from_min_size(egui::Pos2::ZERO, size)
        })
        .unwrap();
        assert!(pixels > 360 * 360);
        sizes.push(pixels);
        assert_eq!(snap.source_pixels_per_point(), 2.0);
        let bytes = snap
            .to_png(ExportOptions {
                pixels_per_point: 2.0,
                ..Default::default()
            })
            .unwrap();
        assert_eq!(
            u32::from_be_bytes(bytes[16..20].try_into().unwrap()),
            width * 2
        );
    }
    assert!(sizes[1] > sizes[0]);
}
