use egui_customplot::{AxisScale, ColorMap, ColorScale, GridData, GridError, PlotBounds};
use std::sync::Arc;

#[test]
fn grid_validates_shape_edges_precision_and_row_major_values() {
    assert_eq!(
        GridData::new(vec![], vec![0.0, 1.0], vec![]).unwrap_err(),
        GridError::EmptyAxis
    );
    for edges in [
        vec![0.0, 0.0],
        vec![2.0, 1.0],
        vec![0.0, f64::INFINITY],
        vec![-1e308, 1e308],
    ] {
        assert_eq!(
            GridData::new(edges, vec![0.0, 1.0], vec![0.0]).unwrap_err(),
            GridError::InvalidEdges
        );
    }
    assert_eq!(
        GridData::new(vec![0.0, 1.0], vec![0.0, 1.0], vec![]).unwrap_err(),
        GridError::LengthMismatch
    );
    let bounds = PlotBounds::new([0.0, 10.0], [2.0, 40.0]).unwrap();
    let grid = GridData::uniform(2, 3, bounds, (0..6).map(f64::from).collect::<Vec<_>>()).unwrap();
    assert_eq!(grid.x_edges(), &[0.0, 1.0, 2.0]);
    assert_eq!(grid.y_edges(), &[10.0, 20.0, 30.0, 40.0]);
    assert_eq!(grid.value(1, 2), Some(5.0));
    assert_eq!(grid.value(2, 0), None);
    assert_eq!(grid.cell_bounds(1, 2).unwrap().min(), [1.0, 30.0]);
    assert!(grid.cell_bounds(0, 3).is_none());
    assert_eq!(
        GridData::uniform(usize::MAX, 2, bounds, vec![]).unwrap_err(),
        GridError::CapacityOverflow
    );
    let tiny = PlotBounds::new([1e16, 0.0], [1e16 + 2.0, 1.0]).unwrap();
    assert_eq!(
        GridData::uniform(4, 1, tiny, vec![1.0; 4]).unwrap_err(),
        GridError::InvalidEdges
    );
}

#[test]
fn grids_share_payload_and_cache_finite_and_positive_color_ranges() {
    let edges: Arc<[f64]> = vec![0.0, 1.0, 2.0].into();
    let values: Arc<[f64]> = vec![f64::NAN, f64::INFINITY, -1.0, 10.0].into();
    let grid = GridData::new(edges.clone(), edges.clone(), values.clone()).unwrap();
    assert_eq!(grid.value_range(AxisScale::Linear), Some([-1.0, 10.0]));
    assert_eq!(grid.value_range(AxisScale::Log10), Some([10.0; 2]));
    assert_eq!(grid.non_finite_cells(), 2);
    let clone = grid.clone();
    assert_eq!(clone.values().as_ptr(), grid.values().as_ptr());
    assert_eq!(Arc::strong_count(&values), 3);
    drop((grid, clone));
    assert_eq!(Arc::strong_count(&values), 1);
    assert_eq!(Arc::strong_count(&edges), 1);
}

#[test]
fn color_limits_are_shared_clamped_and_domain_aware() {
    for range in [[0.0; 2], [1.0, 0.0], [f64::NAN, 1.0], [-1e308, 1e308]] {
        assert!(ColorScale::new(range).is_err());
    }
    assert!(ColorScale::with_scale([0.0, 1.0], AxisScale::Log10).is_err());
    let colors = ColorScale::with_scale([1.0, 100.0], AxisScale::Log10)
        .unwrap()
        .color_map(ColorMap::Gray);
    assert_eq!(colors.fraction(10.0), Some(0.5));
    assert_eq!(colors.color(10.0), Some(egui::Color32::from_gray(128)));
    assert_eq!(colors.color(f64::MAX), Some(egui::Color32::WHITE));
    assert_eq!(colors.color(1e-100), Some(egui::Color32::BLACK));
    for value in [f64::NAN, f64::INFINITY, 0.0, -1.0] {
        assert_eq!(colors.color(value), None);
    }
    let extreme = ColorScale::new([-1e307, 1e307]).unwrap();
    assert_eq!(extreme.fraction(f64::MAX), Some(1.0));
}
