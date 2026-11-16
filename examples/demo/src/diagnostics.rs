use eframe::egui;
use egui_customplot::PlotDiagnostics;

pub(super) fn show(ui: &mut egui::Ui, d: &PlotDiagnostics) {
    if d.patches.input_items != 0 {
        ui.monospace(format!(
            "patches/areas: {} | visited: {} | tris / strokes / meshes: {} / {} / {} | invalid / domain-invalid / baselines: {} / {} / {} | payload: {} B",
            d.patches.input_items,
            d.patches.visited_items,
            d.patches.output_triangles,
            d.patches.output_strokes,
            d.patches.output_meshes,
            d.patches.invalid_items,
            d.patches.domain_invalid_items,
            d.patches.invalid_area_baselines,
            d.patches.unique_bytes
        ));
    }
    if d.contours.input_lines != 0 || d.contours.input_bands != 0 {
        ui.monospace(format!(
            "contours: {} lines, {} bands | visited: {} | paths / labels / meshes: {} / {} / {} | payload: {} B",
            d.contours.input_lines,
            d.contours.input_bands,
            d.contours.visited_items,
            d.contours.output_paths,
            d.contours.output_labels,
            d.contours.output_meshes,
            d.contours.unique_bytes
        ));
    }
    if d.images.input_cells != 0 {
        ui.monospace(format!(
            "image/pcolor cells: {} | visited / painted: {} / {} | nonfinite / domain-invalid: {} / {} | meshes: {} | payload: {} B",
            d.images.input_cells,
            d.images.visited_cells,
            d.images.output_cells,
            d.images.non_finite_cells,
            d.images.domain_invalid_cells,
            d.images.output_meshes,
            d.images.unique_bytes
        ));
    }
    if d.geometry.input_edges != 0
        || d.geometry.input_triangles != 0
        || d.geometry.input_spy_nonzeros != 0
    {
        ui.monospace(format!(
            "geometry: {} tris, {} edges, {} spy nnz | visited: {} | paths / markers / meshes: {} / {} / {} | payload: {} B",
            d.geometry.input_triangles,
            d.geometry.input_edges,
            d.geometry.input_spy_nonzeros,
            d.geometry.visited_items,
            d.geometry.output_paths,
            d.geometry.output_markers,
            d.geometry.output_meshes,
            d.geometry.unique_bytes
        ));
    }
    if d.flow.input_streamlines != 0
        || d.flow.input_feather_vectors != 0
        || d.flow.input_comet_points != 0
    {
        ui.monospace(format!(
            "flow: {} lines, {} feather, {} comet pts | visited: {} | paths / heads: {} / {} | payload: {} B",
            d.flow.input_streamlines,
            d.flow.input_feather_vectors,
            d.flow.input_comet_points,
            d.flow.visited_items,
            d.flow.output_paths,
            d.flow.output_heads,
            d.flow.unique_bytes
        ));
    }
    if d.stats.input_bubbles != 0
        || d.stats.input_boxes != 0
        || d.stats.input_violins != 0
        || d.stats.input_slices != 0
        || d.stats.input_clouds != 0
        || d.stats.input_bins != 0
        || d.stats.input_parallel_rows != 0
    {
        ui.monospace(format!(
            "stats: bubbles {}/{}, boxes {}/{}, violins {}/{}, slices {}/{}, clouds {}/{}, bins {}/{}, parallel {}/{}",
            d.stats.output_bubbles,
            d.stats.input_bubbles,
            d.stats.output_boxes,
            d.stats.input_boxes,
            d.stats.output_violins,
            d.stats.input_violins,
            d.stats.output_slices,
            d.stats.input_slices,
            d.stats.output_clouds,
            d.stats.input_clouds,
            d.stats.output_bins,
            d.stats.input_bins,
            d.stats.output_parallel_segments,
            d.stats.input_parallel_rows,
        ));
    }
    if d.polar.input_points != 0 || d.polar.output_regions != 0 {
        ui.monospace(format!(
            "polar: {} in | visited: {} | paths/markers/sectors/arrows/regions: {}/{}/{}/{}/{} | payload: {} B",
            d.polar.input_points,
            d.polar.visited_items,
            d.polar.output_paths,
            d.polar.output_markers,
            d.polar.output_sectors,
            d.polar.output_arrows,
            d.polar.output_regions,
            d.polar.unique_bytes
        ));
    }
    if d.network.input_nodes != 0 || d.network.input_edges != 0 {
        ui.monospace(format!(
            "network: {} nodes, {} edges (loops={}, multi={}) | out nodes/edges/arrows/labels: {}/{}/{}/{} | payload: {} B",
            d.network.input_nodes,
            d.network.input_edges,
            d.network.self_loops,
            d.network.multi_edges,
            d.network.output_nodes,
            d.network.output_edges,
            d.network.output_arrows,
            d.network.output_labels,
            d.network.unique_bytes
        ));
    }
    if d.plot3d.input_points != 0 || d.plot3d.input_primitives != 0 || d.plot3d.output_planes != 0 {
        ui.monospace(format!(
            "plot3d: {} pts, {} prims | tris/segs/markers/planes: {}/{}/{}/{} | zbuf px/spans: {}/{} | payload: {} B",
            d.plot3d.input_points,
            d.plot3d.input_primitives,
            d.plot3d.output_triangles,
            d.plot3d.output_segments,
            d.plot3d.output_markers,
            d.plot3d.output_planes,
            d.plot3d.zbuffer_pixels,
            d.plot3d.zbuffer_spans,
            d.plot3d.unique_bytes
        ));
    }
    if d.input_quiver_arrows != 0 {
        ui.monospace(format!("vectors: {} | visited: {} | shafts / heads: {} / {} | invalid / domain-invalid: {} / {} | raw pick visits: {} | vector payload: {} B",
            d.input_quiver_arrows, d.visited_quiver_arrows, d.output_quiver_shafts, d.output_quiver_heads,
            d.invalid_quiver_arrows, d.domain_invalid_quiver_arrows, d.visited_pick_quiver_arrows, d.unique_quiver_bytes));
    }
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
