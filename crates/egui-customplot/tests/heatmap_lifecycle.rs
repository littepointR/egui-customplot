mod support;

use egui::{Context, Event, Shape};
use egui_customplot::{ColorMap, ColorScale, GridData, Heatmap, Plot, PlotBounds, PlotState};
use std::sync::Arc;
use support::{capture, run};

#[test]
fn parent_clip_bounds_work_and_hidden_widgets_do_no_cell_work() {
    let ctx = Context::default();
    let data = GridData::uniform(
        100,
        100,
        PlotBounds::new([0.0; 2], [100.0; 2]).unwrap(),
        vec![1.0; 10_000],
    )
    .unwrap();
    let mut state = PlotState::default();
    state.set_bounds(PlotBounds::new([0.0; 2], [100.0; 2]).unwrap());
    let colors = ColorScale::new([0.0, 2.0]).unwrap();
    let first = run(&ctx, vec![], false, |ui| {
        Plot::new("p").show_layers(ui, &mut state, &[Heatmap::new(&data, colors).into()])
    });
    let rect = first.transform.rect();
    let clip = egui::Rect::from_min_max(rect.min, rect.center());
    let (out, output) = capture(&ctx, vec![], false, |ui| {
        ui.set_clip_rect(clip);
        Plot::new("p").show_layers(ui, &mut state, &[Heatmap::new(&data, colors).into()])
    });
    assert!(out.diagnostics.visited_heatmap_cells <= 51 * 51);
    assert!(
        output
            .shapes
            .iter()
            .all(|s| clip.contains_rect(s.clip_rect))
    );
    output.drop_without_applying_deltas();
    let out = run(&ctx, vec![], false, |ui| {
        ui.set_clip_rect(egui::Rect::NOTHING);
        Plot::new("p").show_layers(ui, &mut state, &[Heatmap::new(&data, colors).into()])
    });
    assert!(out.diagnostics.painting_skipped);
    assert_eq!(out.diagnostics.visited_heatmap_cells, 0);
}

#[test]
fn changed_color_mapping_is_fresh_and_data_is_not_retained_by_state_or_context() {
    let ctx = Context::default();
    let mut state = PlotState::default();
    let values: Arc<[f64]> = vec![0.25].into();
    let data = GridData::new(vec![1.0, 2.0], vec![1.0, 2.0], values.clone()).unwrap();
    for map in [ColorMap::Gray, ColorMap::BlueRed, ColorMap::Viridis] {
        let colors = ColorScale::new([0.0, 1.0]).unwrap().color_map(map);
        let (out, output) = capture(&ctx, vec![], true, |ui| {
            Plot::new("p").show_layers(ui, &mut state, &[Heatmap::new(&data, colors).into()])
        });
        assert_eq!(out.diagnostics.output_heatmap_cells, 1);
        let mesh = output
            .shapes
            .iter()
            .find_map(|s| {
                if let Shape::Mesh(m) = &s.shape {
                    Some(m)
                } else {
                    None
                }
            })
            .unwrap();
        assert!(
            mesh.vertices
                .iter()
                .all(|v| Some(v.color) == colors.color(0.25))
        );
        output.drop_without_applying_deltas();
    }
    drop(data);
    assert_eq!(Arc::strong_count(&values), 1);
    assert!(state.bounds().is_none());
}

#[test]
fn high_dpi_and_extreme_offscreen_cells_stay_finite_and_hover_can_be_disabled() {
    let ctx = Context::default();
    let data = GridData::new(
        vec![-1e300, 0.0, 1e300],
        vec![-1e300, 0.0, 1e300],
        vec![1.0; 4],
    )
    .unwrap();
    let colors = ColorScale::new([0.0, 2.0]).unwrap();
    for density in [1.0, 2.0] {
        ctx.set_pixels_per_point(density);
        let mut state = PlotState::default();
        state.set_bounds(PlotBounds::new([-1.0; 2], [1.0; 2]).unwrap());
        let first = run(&ctx, vec![], false, |ui| {
            Plot::new("p").show_layers(ui, &mut state, &[Heatmap::new(&data, colors).into()])
        });
        let pos = first.transform.rect().center();
        let (out, output) = capture(&ctx, vec![Event::PointerMoved(pos)], true, |ui| {
            Plot::new("p").show_hover(false).show_layers(
                ui,
                &mut state,
                &[Heatmap::new(&data, colors).into()],
            )
        });
        assert_eq!(out.diagnostics.output_heatmap_cells, 4);
        assert_eq!(out.diagnostics.heatmap_pick_queries, 0);
        for mesh in output.shapes.iter().filter_map(|s| {
            if let Shape::Mesh(m) = &s.shape {
                Some(m)
            } else {
                None
            }
        }) {
            assert!(
                mesh.vertices
                    .iter()
                    .all(|v| v.pos.is_finite() && out.transform.rect().contains(v.pos))
            );
        }
        output.drop_without_applying_deltas();
    }
}
