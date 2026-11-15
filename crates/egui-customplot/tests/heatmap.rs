mod support;

use egui::{Context, Event, IdSalt, Shape};
use egui_customplot::{
    Axis, AxisScale, ColorMap, ColorScale, GridData, Heatmap, Line, Plot, PlotBounds, PlotData,
    PlotLayer, PlotState, YAxis,
};
use std::sync::Arc;
use support::{capture, run};

fn colors() -> ColorScale {
    ColorScale::new([0.0, 4.0])
        .unwrap()
        .color_map(ColorMap::Gray)
}

#[test]
fn row_major_rectangles_colors_and_missing_cells_reach_the_painter() {
    let ctx = Context::default();
    let data = GridData::new(
        vec![0.0, 1.0, 3.0],
        vec![10.0, 20.0, 40.0],
        vec![0.0, 1.0, f64::NAN, 4.0],
    )
    .unwrap();
    let mut state = PlotState::default();
    let (out, output) = capture(&ctx, vec![], true, |ui| {
        Plot::new("p").show_layers(ui, &mut state, &[Heatmap::new(&data, colors()).into()])
    });
    assert_eq!(
        (
            out.diagnostics.input_heatmap_cells,
            out.diagnostics.non_finite_heatmap_cells
        ),
        (4, 1)
    );
    assert_eq!(
        (
            out.diagnostics.visited_heatmap_cells,
            out.diagnostics.output_heatmap_cells
        ),
        (4, 3)
    );
    let meshes = output
        .shapes
        .iter()
        .filter_map(|s| {
            if let Shape::Mesh(m) = &s.shape {
                Some(m)
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    assert_eq!(meshes.len(), 1);
    let mesh = meshes[0];
    assert_eq!((mesh.vertices.len(), mesh.indices.len()), (12, 18));
    for (quad, (column, row)) in
        mesh.vertices
            .as_chunks::<4>()
            .0
            .iter()
            .zip([(0, 0), (1, 0), (1, 1)])
    {
        let bounds = data.cell_bounds(column, row).unwrap();
        let rect = egui::Rect::from_two_pos(
            out.transform.position_from_point(bounds.min()),
            out.transform.position_from_point(bounds.max()),
        );
        assert!(quad.iter().all(|v| rect.contains(v.pos)
            && v.color == colors().color(data.value(column, row).unwrap()).unwrap()));
        assert_eq!(
            egui::Rect::from_points(&quad.iter().map(|v| v.pos).collect::<Vec<_>>()),
            rect
        );
    }
    assert!(out.transform.bounds().min()[1] < 10.0 && out.transform.bounds().max()[1] > 40.0);
    output.drop_without_applying_deltas();
}

#[test]
fn log_coordinates_color_domains_reversal_and_right_axis_remain_independent() {
    let ctx = Context::default();
    let data = GridData::new(
        vec![-1.0, 1.0, 10.0],
        vec![1.0, 10.0, 100.0],
        vec![1.0, -1.0, 10.0, 100.0],
    )
    .unwrap();
    let mut state = PlotState::default();
    let colors = ColorScale::with_scale([1.0, 100.0], AxisScale::Log10).unwrap();
    let layer = Heatmap::new(&data, colors).y_axis(YAxis::Right);
    let plot = || {
        Plot::new("p")
            .x_axis(Axis::new().scale(AxisScale::Log10).reversed(true))
            .right_y_axis(Axis::new().scale(AxisScale::Log10).reversed(true))
    };
    let first = run(&ctx, vec![], false, |ui| {
        plot().show_layers(ui, &mut state, &[layer.into()])
    });
    assert_eq!(first.diagnostics.domain_invalid_heatmap_cells, 2);
    assert_eq!(first.diagnostics.output_heatmap_cells, 1);
    let right = first.right_transform.unwrap();
    let pos = right.position_from_point([2.0, 20.0]);
    let out = run(&ctx, vec![Event::PointerMoved(pos)], true, |ui| {
        plot().show_layers(ui, &mut state, &[layer.into()])
    });
    let cell = out.hovered_cell.unwrap();
    assert_eq!(
        (cell.column, cell.row, cell.value, cell.y_axis),
        (1, 1, 100.0, YAxis::Right)
    );
    assert!(out.hover_text.unwrap().contains("1.00000000e2"));
    assert_eq!(out.diagnostics.heatmap_pick_queries, 1);
}

#[test]
fn hidden_grids_share_bytes_and_never_freeze_point_series_visibility() {
    let ctx = Context::default();
    let edges: Arc<[f64]> = vec![100.0, 200.0].into();
    let values: Arc<[f64]> = vec![1.0].into();
    let data = GridData::new(edges.clone(), edges, values).unwrap();
    let clone = data.clone();
    let points = PlotData::new(vec![[0.0; 2], [1.0; 2]]);
    let mut state = PlotState::default();
    state.set_series_visible(IdSalt::new("a"), false);
    state.set_series_visible(IdSalt::new("b"), false);
    let out = run(&ctx, vec![], false, |ui| {
        Plot::new("p").show_layers(
            ui,
            &mut state,
            &[
                Heatmap::new(&data, colors()).id_salt("a").into(),
                Heatmap::new(&clone, colors()).id_salt("b").into(),
                Line::new(&points).into(),
            ],
        )
    });
    assert_eq!(out.diagnostics.unique_grid_bytes, 3 * 8);
    assert_eq!(out.diagnostics.visible_series, 1);
    assert_eq!(out.diagnostics.visited_heatmap_cells, 0);
    assert!(out.transform.bounds().max()[0] < 2.0);
    assert_eq!(out.diagnostics.output_paths, 1);
}

#[test]
fn small_view_queries_only_visible_cells_and_meshes_are_chunked_without_lod() {
    let ctx = Context::default();
    let data = GridData::uniform(
        1000,
        1000,
        PlotBounds::new([0.0; 2], [1000.0; 2]).unwrap(),
        vec![1.0; 1_000_000],
    )
    .unwrap();
    let mut state = PlotState::default();
    state.set_bounds(PlotBounds::new([400.25; 2], [402.75; 2]).unwrap());
    let out = run(&ctx, vec![], false, |ui| {
        Plot::new("p").show_layers(ui, &mut state, &[Heatmap::new(&data, colors()).into()])
    });
    assert_eq!(out.diagnostics.visited_heatmap_cells, 9);
    assert_eq!(out.diagnostics.output_heatmap_cells, 9);
    let data = GridData::uniform(130, 130, PlotBounds::default(), vec![1.0; 16_900]).unwrap();
    state.reset();
    let (out, output) = capture(&ctx, vec![], false, |ui| {
        Plot::new("p").show_layers(ui, &mut state, &[Heatmap::new(&data, colors()).into()])
    });
    assert_eq!(out.diagnostics.output_heatmap_cells, 16_900);
    assert_eq!(out.diagnostics.output_heatmap_meshes, 2);
    for mesh in output.shapes.iter().filter_map(|s| {
        if let Shape::Mesh(m) = &s.shape {
            Some(m)
        } else {
            None
        }
    }) {
        assert!(mesh.vertices.len() <= 16_384 * 4);
        assert!(mesh.is_valid());
    }
    output.drop_without_applying_deltas();
}

#[test]
fn grid_hover_is_half_open_and_filled_layers_occlude_only_earlier_points() {
    let ctx = Context::default();
    let grid = GridData::new(vec![0.0, 1.0, 2.0], vec![0.0, 1.0], vec![1.0, f64::NAN]).unwrap();
    let points = PlotData::new(vec![[0.5, 0.5]]);
    for grid_first in [false, true] {
        let mut layers: [PlotLayer<'_>; 2] = [
            Line::new(&points).into(),
            Heatmap::new(&grid, colors()).into(),
        ];
        if grid_first {
            layers.reverse();
        }
        let mut state = PlotState::default();
        let first = run(&ctx, vec![], false, |ui| {
            Plot::new("p").show_layers(ui, &mut state, &layers)
        });
        let pos = first.transform.position_from_point(points.points()[0]);
        let out = run(&ctx, vec![Event::PointerMoved(pos)], true, |ui| {
            Plot::new("p").show_layers(ui, &mut state, &layers)
        });
        assert_eq!(out.hovered_point.is_some(), grid_first);
        assert_eq!(out.hovered_cell.is_some(), !grid_first);
    }
    let mut state = PlotState::default();
    state.set_bounds(PlotBounds::new([0.0; 2], [2.0, 1.0]).unwrap());
    let initial = run(&ctx, vec![], false, |ui| {
        Plot::new("p").show_layers(ui, &mut state, &[Heatmap::new(&grid, colors()).into()])
    });
    let pos = initial.transform.position_from_point([1.0, 0.5]);
    let out = run(&ctx, vec![Event::PointerMoved(pos)], false, |ui| {
        Plot::new("p").show_layers(ui, &mut state, &[Heatmap::new(&grid, colors()).into()])
    });
    assert!(
        out.hovered_cell.is_none(),
        "the missing cell to the right owns the shared edge"
    );
}
