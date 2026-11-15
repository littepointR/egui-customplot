mod support;

use std::sync::Arc;

use egui::{Color32, Context, Event, Shape, Stroke};
use egui_customplot::{
    Line, Plot, PlotData, PlotState, PlotStyle, PlotTheme, Scatter, SeriesStyle,
};
use support::{capture, run};

fn strokes(output: &egui::FullOutput) -> Vec<egui::epaint::PathStroke> {
    output
        .shapes
        .iter()
        .filter_map(|s| match &s.shape {
            Shape::Path(path) => Some(path.stroke.clone()),
            _ => None,
        })
        .collect()
}

#[test]
fn theme_parent_plot_and_series_precedence_reaches_painter() {
    let ctx = Context::default();
    let data = PlotData::new(vec![[0.0; 2], [1.0; 2]]);
    let mut state = PlotState::default();
    let mut theme = PlotTheme::scientific_light();
    theme.style.series.color = Some(Color32::RED);
    theme.style.series.line_width = Some(2.0);
    let theme = Arc::new(theme);
    for (parent, local, series, expected) in [
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
        let (_, out) = capture(&ctx, vec![], false, |ui| {
            Plot::new("p")
                .theme(theme.clone())
                .inherited_style(PlotStyle {
                    series: SeriesStyle {
                        color: parent,
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
                .show(
                    ui,
                    &mut state,
                    &[Line::new(&data).style(SeriesStyle {
                        color: series,
                        ..Default::default()
                    })],
                )
        });
        assert_eq!(
            strokes(&out),
            vec![egui::epaint::PathStroke::new(2.0, expected)]
        );
        out.drop_without_applying_deltas();
    }
}

#[test]
fn width_only_preserves_palette_and_empty_palette_is_safe() {
    let ctx = Context::default();
    let data = PlotData::new(vec![[0.0; 2], [1.0; 2]]);
    for empty in [false, true] {
        let mut theme = PlotTheme::monitoring_dark();
        if empty {
            theme.palette = Arc::from([]);
        }
        let theme = Arc::new(theme);
        let mut state = PlotState::default();
        let mut expected = Vec::new();
        for width in [None, Some(4.0)] {
            let (_, out) = capture(&ctx, vec![], false, |ui| {
                let a = Line::new(&data);
                let b = Line::new(&data);
                Plot::new("p").theme(theme.clone()).show(
                    ui,
                    &mut state,
                    &[
                        width.map_or(a, |w| a.width(w)),
                        width.map_or(b, |w| b.width(w)),
                    ],
                )
            });
            let actual = strokes(&out);
            assert_eq!(actual.len(), 2);
            if width.is_none() {
                expected = actual.iter().map(|s| s.color.clone()).collect();
            } else {
                assert_eq!(
                    actual.iter().map(|s| s.color.clone()).collect::<Vec<_>>(),
                    expected
                );
                assert!(actual.iter().all(|s| s.width == 4.0));
            }
            out.drop_without_applying_deltas();
        }
        assert_ne!(expected[0], expected[1]);
    }
}

#[test]
fn transparent_zero_and_false_override_inherited_values_for_render_and_pick() {
    let ctx = Context::default();
    let data = PlotData::new(vec![[0.0; 2], [1.0; 2]]);
    let inherited = PlotStyle {
        series: SeriesStyle {
            line_width: Some(3.0),
            line_markers: Some(true),
            marker_radius: Some(8.0),
            ..Default::default()
        },
        ..Default::default()
    };
    let mut state = PlotState::default();
    let first = run(&ctx, vec![], false, |ui| {
        Plot::new("p").show(ui, &mut state, &[Line::new(&data)])
    });
    let pos = first.transform.position_from_point([0.0; 2]);
    for line in [
        Line::new(&data).color(Color32::TRANSPARENT),
        Line::new(&data).width(0.0).show_markers(false),
        Line::new(&data).width(0.0).style(SeriesStyle {
            marker_radius: Some(0.0),
            ..Default::default()
        }),
    ] {
        let out = run(&ctx, vec![Event::PointerMoved(pos)], true, |ui| {
            Plot::new("p")
                .inherited_style(inherited)
                .show(ui, &mut state, &[line])
        });
        assert_eq!(out.diagnostics.output_paths, 0);
        assert_eq!(out.diagnostics.output_markers, 0);
        assert!(out.hovered_point.is_none());
    }
    let out = run(&ctx, vec![], false, |ui| {
        Plot::new("p").inherited_style(inherited).show(
            ui,
            &mut state,
            &[Line::new(&data).width(0.0)],
        )
    });
    assert_eq!(out.diagnostics.output_markers, 2);
    assert!(out.hovered_point.is_some());
}

#[test]
fn scatter_point_overrides_take_priority_over_transparent_zero_defaults() {
    let ctx = Context::default();
    let data = PlotData::new(vec![[1.0; 2]]);
    let mut state = PlotState::default();
    let initial = run(&ctx, vec![], false, |ui| {
        Plot::new("p").show(ui, &mut state, &[Line::new(&data)])
    });
    let position = initial.transform.position_from_point([1.0; 2]);
    let out = run(&ctx, vec![Event::PointerMoved(position)], false, |ui| {
        Plot::new("p")
            .style(PlotStyle {
                series: SeriesStyle {
                    color: Some(Color32::TRANSPARENT),
                    marker_radius: Some(0.0),
                    ..Default::default()
                },
                ..Default::default()
            })
            .show_series(
                ui,
                &mut state,
                &[Scatter::new(&data)
                    .colors(&[Color32::RED])
                    .unwrap()
                    .radii(&[5.0])
                    .unwrap()
                    .into()],
            )
    });
    assert_eq!(out.diagnostics.output_markers, 1);
    assert_eq!(out.hovered_point.unwrap().point_index, 0);
}

#[test]
fn themes_do_not_mutate_host_or_neighbor_plot_style() {
    let ctx = Context::default();
    let data = PlotData::new(vec![[0.0; 2], [1.0; 2]]);
    let mut states: [PlotState; 3] = Default::default();
    let mut host = None;
    let mut colors = Vec::new();
    let out = ctx.run_ui(support::input(vec![]), |ui| {
        let before = ui.style().clone();
        host = Some(ui.visuals().extreme_bg_color);
        for (i, theme) in [
            Some(PlotTheme::scientific_light()),
            Some(PlotTheme::monitoring_dark()),
            None,
        ]
        .into_iter()
        .enumerate()
        {
            let plot = Plot::new(i).height(140.0);
            let plot = theme.map_or(plot, |theme| Plot::new(i).height(140.0).theme(theme));
            colors.push(
                plot.show(ui, &mut states[i], &[Line::new(&data)])
                    .transform
                    .rect(),
            );
        }
        assert_eq!(ui.style(), &before);
    });
    for (rect, color) in
        colors
            .iter()
            .zip([Color32::WHITE, Color32::from_rgb(12, 18, 25), host.unwrap()])
    {
        assert!(
            out.shapes
                .iter()
                .any(|s| matches!(&s.shape, Shape::Rect(r) if r.rect == *rect && r.fill == color))
        );
    }
    out.drop_without_applying_deltas();
}

#[test]
fn optional_layers_preserve_explicit_plot_values_and_builder_order() {
    let base = PlotStyle {
        background: Some(Color32::RED),
        grid_stroke: Some(Stroke::new(4.0, Color32::BLUE)),
        series: SeriesStyle {
            line_markers: Some(true),
            ..Default::default()
        },
        ..Default::default()
    };
    let merged = PlotStyle {
        background: Some(Color32::TRANSPARENT),
        grid_stroke: Some(Stroke::NONE),
        series: SeriesStyle {
            line_markers: Some(false),
            ..Default::default()
        },
        ..Default::default()
    }
    .over(base);
    assert_eq!(merged.background, Some(Color32::TRANSPARENT));
    assert_eq!(merged.grid_stroke, Some(Stroke::NONE));
    assert_eq!(merged.series.line_markers, Some(false));
    let ctx = Context::default();
    let mut state = PlotState::default();
    let (_, out) = capture(&ctx, vec![], false, |ui| {
        Plot::new("p")
            .style(base)
            .background(Color32::GREEN)
            .style(PlotStyle::default())
            .show(ui, &mut state, &[])
    });
    assert!(
        out.shapes
            .iter()
            .any(|s| matches!(&s.shape, Shape::Rect(r) if r.fill == Color32::GREEN))
    );
    out.drop_without_applying_deltas();
}

#[test]
fn invalid_plot_stroke_widths_do_not_reach_the_painter() {
    let ctx = Context::default();
    let mut state = PlotState::default();
    for width in [f32::NAN, f32::INFINITY, -1.0] {
        let stroke = Some(Stroke::new(width, Color32::RED));
        let (result, out) = capture(&ctx, vec![], false, |ui| {
            Plot::new("p")
                .style(PlotStyle {
                    grid_stroke: stroke,
                    minor_grid_stroke: stroke,
                    border_stroke: stroke,
                    crosshair_stroke: stroke,
                    ..Default::default()
                })
                .show(ui, &mut state, &[])
        });
        assert_eq!(result.diagnostics.numeric_rejections, 4);
        for s in &out.shapes {
            match &s.shape {
                Shape::LineSegment { stroke, .. } => {
                    assert!(stroke.width.is_finite() && stroke.width >= 0.0)
                }
                Shape::Rect(rect) => {
                    assert!(rect.stroke.width.is_finite() && rect.stroke.width >= 0.0)
                }
                _ => {}
            }
        }
        out.drop_without_applying_deltas();
    }
}
