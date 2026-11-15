mod support;

use egui::{Color32, Context, Event, Stroke};
use egui_customplot::{
    Feather, FeatherData, Plot, PlotBounds, PlotState, Spy, SpyData, SpyError, StreamOptions,
    StreamlineData, Streamlines, VectorField2D,
};
use support::run;

#[test]
fn streamline_rk4_circular_orbit_and_streamslice_arrows() {
    let n = 41usize;
    let bounds = PlotBounds::new([-2.0, -2.0], [2.0, 2.0]).unwrap();
    let mut u = Vec::with_capacity(n * n);
    let mut v = Vec::with_capacity(n * n);
    for iy in 0..n {
        let y = -2.0 + 4.0 * (iy as f64 / (n - 1) as f64);
        for ix in 0..n {
            let x = -2.0 + 4.0 * (ix as f64 / (n - 1) as f64);
            u.push(-y);
            v.push(x);
        }
    }
    let field = VectorField2D::uniform(n, n, bounds, u, v).unwrap();

    let circular = StreamlineData::from_seeds(
        &field,
        &[[1.0, 0.0]],
        StreamOptions {
            step_fraction: 0.15,
            max_steps: 2500,
            bidirectional: false,
            min_speed: 1e-9,
        },
    )
    .unwrap();
    assert_eq!(circular.lines().len(), 1);
    assert!(circular.closed_loops() >= 1);
    for &[x, y] in circular.lines()[0].iter() {
        let r = x.hypot(y);
        assert!(
            (r - 1.0).abs() < 5e-3,
            "RK4 circular orbit drifted from radius 1: r={r}"
        );
    }

    let bounded = StreamlineData::from_seeds(
        &field,
        &[[1.0, 0.0]],
        StreamOptions {
            max_steps: 5,
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(bounded.budget_stops(), 1);

    let slice = StreamlineData::streamslice(&field, 1.0).unwrap();
    assert!(!slice.lines().is_empty());
    assert!(!slice.arrows().is_empty());

    let ctx = Context::default();
    let mut state = PlotState::default();
    state.set_bounds(bounds);
    let resp = run(&ctx, vec![], true, |ui| {
        Plot::new("streamslice").show_layers(ui, &mut state, &[Streamlines::new(&slice).into()])
    });
    assert!(resp.diagnostics.flow.output_paths >= slice.lines().len());
    assert!(resp.diagnostics.flow.output_heads > 0);
}

#[test]
fn feather_constructors_baseline_and_vector_picking() {
    let f1 = FeatherData::from_components(&[0.5, 0.0, -0.5], &[1.0, 1.2, 0.8]).unwrap();
    assert_eq!(f1.len(), 3);
    let f2 = FeatherData::from_complex(&[[1.0, 0.5], [0.0, -0.8]]);
    assert_eq!(f2.len(), 2);
    let f3 = FeatherData::from_polar(&[0.0, std::f64::consts::FRAC_PI_2], &[1.0, 2.0])
        .unwrap()
        .with_baseline(10.0, 2.0, 1.0);
    assert_eq!(f3.baseline_y(), 1.0);
    let tip0 = f3.quiver().arrows()[0].endpoint().unwrap();
    assert!((tip0[0] - 11.0).abs() < 1e-12 && (tip0[1] - 1.0).abs() < 1e-12);

    let ctx = Context::default();
    let mut state = PlotState::default();
    state.set_bounds(PlotBounds::new([9.0, -0.5], [14.0, 4.0]).unwrap());
    let first = run(&ctx, vec![], false, |ui| {
        Plot::new("feather").show_layers(
            ui,
            &mut state,
            &[Feather::new(&f3)
                .baseline_stroke(Stroke::new(1.0, Color32::GRAY))
                .into()],
        )
    });
    assert!(first.diagnostics.flow.output_paths >= 3);
    assert_eq!(first.diagnostics.flow.output_heads, 2);

    let anchor_pos = first.transform.position_from_point(tip0);
    let hovered = run(&ctx, vec![Event::PointerMoved(anchor_pos)], true, |ui| {
        Plot::new("feather").show_layers(ui, &mut state, &[Feather::new(&f3).into()])
    });
    let vec_hit = hovered.hovered_vector.expect("expected hovered vector");
    assert_eq!(vec_hit.arrow_index, 0);
}

#[test]
fn spy_csr_window_culling_spy_axes_and_picking() {
    assert_eq!(
        SpyData::from_coords(4, 4, &[(4, 0)]).unwrap_err(),
        SpyError::IndexOutOfBounds {
            row: 4,
            col: 0,
            rows: 4,
            cols: 4
        }
    );

    let from_dense = SpyData::from_dense(2, 3, &[1.0, 0.0, -2.0, 0.0, 0.0, 0.5], 0.0).unwrap();
    assert_eq!(from_dense.nnz(), 3);

    // 1000x1000 banded matrix (~2000 nonzeros + 1 duplicate entry)
    let n = 1000usize;
    let mut entries = Vec::with_capacity(n * 2);
    for i in 0..n {
        entries.push((i, i));
        if i + 1 < n {
            entries.push((i, i + 1));
        }
    }
    entries.push((10, 10)); // duplicate
    let spy = SpyData::from_coords(n, n, &entries).unwrap();
    assert_eq!(spy.duplicate_entries(), 1);
    assert_eq!(spy.nnz(), n * 2 - 1);

    let csr = SpyData::from_csr(2, 2, &[0, 1, 2], &[1, 0]).unwrap();
    assert_eq!(csr.nnz(), 2);

    let ctx = Context::default();
    let mut state = PlotState::default();
    state.set_bounds(PlotBounds::new([9.0, 9.0], [21.0, 21.0]).unwrap());
    let zoomed = run(&ctx, vec![], false, |ui| {
        Plot::new("spy")
            .spy_axes()
            .show_layers(ui, &mut state, &[Spy::new(&spy).into()])
    });
    assert!(zoomed.transform.reversed()[1]);
    assert_eq!(zoomed.transform.data_aspect(), Some(1.0));
    assert!(
        zoomed.diagnostics.geometry.visited_items < 40,
        "expected CSR window culling, visited={}",
        zoomed.diagnostics.geometry.visited_items
    );
    assert!(zoomed.diagnostics.geometry.output_markers > 0);

    // Coordinate (1-based) for (row=9, col=9) is [10.0, 10.0]
    let pos = zoomed.transform.position_from_point([10.0, 10.0]);
    let picked = run(&ctx, vec![Event::PointerMoved(pos)], true, |ui| {
        Plot::new("spy")
            .spy_axes()
            .show_layers(ui, &mut state, &[Spy::new(&spy).into()])
    });
    assert_eq!(picked.hovered_point.unwrap().point, [10.0, 10.0]);
}
