use eframe::egui;
use egui_customplot::PlotDiagnostics;

pub(super) fn show(ui: &mut egui::Ui, d: &PlotDiagnostics) {
    if d.input_bars != 0 || d.input_error_bars != 0 {
        ui.monospace(format!("bars: {} (painted {}) | errors: {} (segments {}) | invalid / domain-invalid: {} / {} | primitive payload: {} B",
            d.input_bars, d.output_bars, d.input_error_bars, d.output_error_bar_segments,
            d.invalid_bars + d.invalid_error_bars, d.domain_invalid_bars + d.domain_invalid_error_bars,
            d.unique_bar_bytes + d.unique_error_bar_bytes));
    }
    if d.input_heatmap_cells != 0 {
        ui.monospace(format!("cells: {} | visited / painted: {} / {} | missing / color-invalid / axis-invalid: {} / {} / {} | grid: {} B | meshes: {}",
            d.input_heatmap_cells, d.visited_heatmap_cells, d.output_heatmap_cells, d.non_finite_heatmap_cells,
            d.color_invalid_heatmap_cells, d.domain_invalid_heatmap_cells, d.unique_grid_bytes, d.output_heatmap_meshes));
    }
    ui.monospace(format!(
        "samples: {} | segments visited: {} | markers: {} / {} | pick visits: {}",
        d.input_points,
        d.visited_segments,
        d.output_markers,
        d.visited_marker_points,
        d.visited_pick_points
    ));
    ui.monospace(format!("sample payload: {:.2} MiB | geometry capacity: {:.2} MiB | prepare: {:.3} ms | pick: {:.3} ms",
        d.unique_sample_bytes as f64 / 1_048_576.0, d.geometry_capacity_bytes as f64 / 1_048_576.0,
        d.prepare_duration.as_secs_f64() * 1000.0, d.pick_duration.as_secs_f64() * 1000.0));
    ui.monospace(format!(
        "nonfinite: {} | log-domain invalid: {} | numeric rejections: {} | visible series: {}",
        d.non_finite_points, d.domain_invalid_points, d.numeric_rejections, d.visible_series
    ));
    ui.monospace(format!("indexed lines: {} | fallback: {} | selected: {} | runs: {} | comparisons: {} | tree nodes: {} | scans: {}",
        d.indexed_line_series, d.lod_fallback_series, d.query_selected_points, d.query_runs, d.query_comparisons, d.query_index_nodes, d.query_scanned_points));
    ui.monospace(format!("index: {} B | query scratch: {} B | query: {:.3} ms | cache hit/miss/rejected: {}/{}/{} | retained: {} B",
        d.unique_index_bytes, d.query_capacity_bytes, d.query_duration.as_secs_f64() * 1000.0,
        d.line_cache_hits, d.line_cache_misses, d.line_cache_rejections, d.line_cache_bytes));
    ui.small("Per-pass CPU counters only. Excludes tessellation, process/GPU memory and presentation. Line LOD/cache do not optimize raw markers or guarantee FPS.");
    ui.small(format!(
        "Annotations submitted: {} | invalid: {}",
        d.output_annotations, d.annotation_rejections
    ));
    if d.pattern_truncated || d.aspect_constraint_failed {
        ui.colored_label(
            egui::Color32::YELLOW,
            format!(
                "Safety diagnostics: dash cap={}, unrepresentable aspect={}",
                d.pattern_truncated, d.aspect_constraint_failed
            ),
        );
    }
}
