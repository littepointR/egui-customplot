mod support;

use egui::{Color32, Context, Shape};
use egui_customplot::{
    Figure, FigureError, Line, Plot, PlotData, PlotState, PlotStyle, PlotTheme, SeriesStyle,
};
use support::run;

#[test]
fn row_major_layout_shares_defaults_not_view_state_or_widget_ids() {
    let ctx = Context::default();
    let data = PlotData::new(vec![[1.0; 2], [2.0; 2]]);
    let mut states: [PlotState; 4] = Default::default();
    let mut rects = Vec::new();
    let mut ids = std::collections::HashSet::new();
    let output = ctx.run_ui(support::input(vec![]), |ui| {
        let host_style = ui.style().clone();
        let figure = Figure::new("f")
            .columns(2)
            .cell_height(180.0)
            .theme(PlotTheme::scientific_light())
            .style(PlotStyle {
                series: SeriesStyle {
                    color: Some(Color32::GREEN),
                    line_width: Some(3.0),
                    ..Default::default()
                },
                ..Default::default()
            })
            .show(ui, 4, |cell| {
                let i = cell.index();
                assert_eq!((cell.row(), cell.column()), (i / 2, i % 2));
                let plot = if i == 1 {
                    Plot::new("p").style(PlotStyle {
                        series: SeriesStyle {
                            color: Some(Color32::RED),
                            ..Default::default()
                        },
                        ..Default::default()
                    })
                } else {
                    Plot::new("p")
                };
                cell.show_plot(
                    plot,
                    &mut states[i],
                    &[if i == 2 {
                        Line::new(&data).color(Color32::GOLD)
                    } else {
                        Line::new(&data)
                    }],
                )
            })
            .unwrap();
        for response in figure.inner {
            assert!(ids.insert(response.response.id));
            assert!(response.transform.rect().is_positive());
        }
        rects = figure.cell_rects;
        assert_eq!(ui.style(), &host_style);
    });
    assert!(rects[0].right() < rects[1].left());
    assert!(rects[0].bottom() < rects[2].top());
    assert_eq!(rects[0].size(), rects[3].size());
    let strokes = output
        .shapes
        .iter()
        .filter_map(|s| match &s.shape {
            Shape::Path(p) => Some(p.stroke.clone()),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(
        strokes.iter().map(|s| s.color.clone()).collect::<Vec<_>>(),
        [Color32::GREEN, Color32::RED, Color32::GOLD, Color32::GREEN]
            .map(egui::epaint::ColorMode::Solid)
    );
    assert!(strokes.iter().all(|s| s.width == 3.0));
    assert!(states.iter().all(|s| s.bounds().is_none()));
    output.drop_without_applying_deltas();
}

#[test]
fn cells_obey_parent_clip_and_do_not_prepare_fully_hidden_plots() {
    let ctx = Context::default();
    let data = PlotData::new(vec![[0.0; 2], [1.0; 2]]);
    let mut states: [PlotState; 4] = Default::default();
    let clip = egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(300.0, 150.0));
    let output = ctx.run_ui(support::input(vec![]), |ui| {
        ui.set_clip_rect(clip);
        let figure = Figure::new("f")
            .columns(2)
            .cell_height(200.0)
            .show(ui, 4, |cell| {
                cell.show_plot(
                    Plot::new("p"),
                    &mut states[cell.index()],
                    &[Line::new(&data)],
                )
            })
            .unwrap();
        assert!(!figure.inner[0].diagnostics.painting_skipped);
        assert!(
            figure.inner[1..]
                .iter()
                .all(|r| r.diagnostics.painting_skipped)
        );
        assert!(
            figure.inner[1..]
                .iter()
                .all(|r| r.diagnostics.visited_segments == 0)
        );
    });
    for shape in &output.shapes {
        assert!(clip.contains_rect(shape.clip_rect));
    }
    output.drop_without_applying_deltas();
}

#[test]
fn empty_limits_small_parents_and_layout_retries_are_explicit() {
    let ctx = Context::default();
    let mut state = PlotState::default();
    run(&ctx, vec![], true, |ui| {
        assert!(
            Figure::new("empty")
                .show(ui, 0, |_| panic!("empty cell"))
                .unwrap()
                .inner
                .is_empty()
        );
        assert!(matches!(
            Figure::new("too-many").show(ui, 257, |_| panic!("rejected cell")),
            Err(FigureError::TooManyCells {
                requested: 257,
                maximum: 256
            })
        ));
        ui.set_width(1.0);
        let mut figure = Figure::new("tiny")
            .columns(0)
            .cell_height(1.0)
            .spacing(egui::vec2(100.0, 100.0))
            .show(ui, 1, |cell| {
                cell.show_plot(Plot::new("p"), &mut state, &[])
            })
            .unwrap();
        assert!(figure.cell_rects[0].is_finite());
        figure.inner.pop().unwrap()
    });
}

#[test]
fn cells_follow_the_allocated_rectangle_in_directional_and_justified_parents() {
    let ctx = Context::default();
    for layout in [
        egui::Layout::bottom_up(egui::Align::Center),
        egui::Layout::right_to_left(egui::Align::Center),
        egui::Layout::top_down(egui::Align::Center).with_main_justify(true),
        egui::Layout::left_to_right(egui::Align::Center).with_cross_justify(true),
    ] {
        ctx.run_ui(support::input(vec![]), |ui| {
            ui.with_layout(layout, |ui| {
                let figure = Figure::new("directional")
                    .columns(2)
                    .cell_height(60.0)
                    .show(ui, 3, |cell| cell.rect())
                    .unwrap();
                assert_eq!(figure.inner, figure.cell_rects);
                assert!(
                    figure
                        .cell_rects
                        .iter()
                        .all(|rect| figure.response.rect.contains_rect(*rect))
                );
                let union = figure
                    .cell_rects
                    .iter()
                    .copied()
                    .reduce(egui::Rect::union)
                    .unwrap();
                assert_eq!(
                    union.size(),
                    egui::vec2(figure.cell_rects[0].width() * 2.0 + 8.0, 128.0)
                );
                assert_eq!(union.bottom(), figure.cell_rects[2].bottom());
            });
        })
        .drop_without_applying_deltas();
    }
}
