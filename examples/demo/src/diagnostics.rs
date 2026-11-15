use eframe::egui;
use egui_customplot::PlotDiagnostics;

pub(super) fn show(ui: &mut egui::Ui, d: &PlotDiagnostics) {
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
    ui.small("Per-pass CPU counters only. Excludes tessellation, process/GPU memory and presentation. No LOD, geometry cache or performance guarantee.");
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
