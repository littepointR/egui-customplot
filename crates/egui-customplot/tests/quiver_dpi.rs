mod support;

use egui::{Color32, Context, Shape};
use egui_customplot::{
    Plot, PlotBounds, PlotState, PlotStyle, PlotTheme, Quiver, QuiverArrow, QuiverData, SeriesStyle,
};
use support::{capture, run};

#[test]
fn head_dimensions_are_logical_points_at_multiple_host_dpi_and_resized_views() {
    let data = QuiverData::new(vec![QuiverArrow::new([0.2; 2], [0.6, 0.0])]);
    for dpi in [1.0, 1.5, 2.0] {
        let ctx = Context::default();
        ctx.set_pixels_per_point(dpi);
        for height in [200.0, 400.0] {
            let mut state = PlotState::default();
            state.set_bounds(PlotBounds::new([0.0; 2], [1.0; 2]).unwrap());
            // Apply the host DPI change before measuring a stable layout.
            run(&ctx, vec![], false, |ui| {
                Plot::new("p")
                    .height(height)
                    .show_layers(ui, &mut state, &[])
            });
            let (_, output) = capture(&ctx, vec![], true, |ui| {
                Plot::new("p").height(height).show_layers(
                    ui,
                    &mut state,
                    &[Quiver::new(&data).head_size(12.0, 10.0).into()],
                )
            });
            assert_eq!(output.pixels_per_point, dpi);
            let head = output
                .shapes
                .iter()
                .find_map(|s| match &s.shape {
                    Shape::Path(p) if p.points.len() == 3 => Some(p.points.clone()),
                    _ => None,
                })
                .unwrap();
            output.drop_without_applying_deltas();
            assert!((head[1].x - head[0].x - 12.0).abs() < 0.0001);
            assert!((head[0].y - head[2].y - 10.0).abs() < 0.0001);
        }
    }
}

#[test]
fn theme_parent_plot_and_vector_overrides_style_both_parts_without_adding_point_markers() {
    let ctx = Context::default();
    let data = QuiverData::new(vec![QuiverArrow::new([1.0; 2], [2.0; 2])]);
    let mut theme = PlotTheme::scientific_light();
    theme.style.series.color = Some(Color32::RED);
    theme.style.series.line_width = Some(3.0);
    let theme = std::sync::Arc::new(theme);
    for (parent, local, vector, expected) in [
        (None, None, None, Color32::RED),
        (Some(Color32::GREEN), None, None, Color32::GREEN),
        (
            Some(Color32::GREEN),
            Some(Color32::BLUE),
            None,
            Color32::BLUE,
        ),
        (
            Some(Color32::GREEN),
            Some(Color32::BLUE),
            Some(Color32::GOLD),
            Color32::GOLD,
        ),
    ] {
        let mut state = PlotState::default();
        let (out, output) = capture(&ctx, vec![], false, |ui| {
            Plot::new("p")
                .theme(theme.clone())
                .inherited_style(PlotStyle {
                    series: SeriesStyle {
                        color: parent,
                        line_markers: Some(true),
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .style(PlotStyle {
                    series: SeriesStyle {
                        color: local,
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .show_layers(
                    ui,
                    &mut state,
                    &[Quiver::new(&data)
                        .style(SeriesStyle {
                            color: vector,
                            ..Default::default()
                        })
                        .into()],
                )
        });
        let strokes: Vec<_> = output
            .shapes
            .iter()
            .filter_map(|s| match &s.shape {
                Shape::Path(p) => Some(p.stroke.clone()),
                _ => None,
            })
            .collect();
        output.drop_without_applying_deltas();
        assert_eq!(
            strokes,
            vec![egui::epaint::PathStroke::new(3.0, expected); 2]
        );
        assert_eq!(out.diagnostics.output_markers, 0);
    }
}
