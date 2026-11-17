#![cfg(feature = "export")]

use egui::{Color32, Context, Event, Modifiers, PointerButton, Pos2, RawInput, Rect};
use egui_customplot::{
    Axis, BarOrientation, BoxChart, BoxChartData, ColorMap, ColorScale, ExportOptions, Figure,
    GridData, Heatmap, Line, MarkerShape, MarkerStyle, Plot, PlotBounds, PlotData, PlotLayer,
    PlotSnapshot, PlotState, Raincloud, RaincloudData, Scatter, ViolinData, ViolinOptions,
    ViolinPlot, civil_to_unix_secs,
};

fn input(events: Vec<Event>) -> RawInput {
    RawInput {
        screen_rect: Some(Rect::from_min_size(Pos2::ZERO, egui::vec2(960.0, 680.0))),
        events,
        ..Default::default()
    }
}

#[test]
fn stage12_foundation_features_export_self_contained_svg_and_png_with_true_hollow_center() {
    let t0 = civil_to_unix_secs(2026, 6, 1, 8, 0, 0.0).unwrap();
    let dt_pts = PlotData::new(vec![
        [t0, 1.0],
        [t0 + 1800.0, 2.5],
        [t0 + 3600.0, 1.4],
        [t0 + 7200.0, 3.1],
    ]);
    let grid = GridData::uniform(
        16,
        12,
        PlotBounds::new([0.0, 0.0], [4.0, 3.0]).unwrap(),
        (0..192).map(|i| (i as f64) / 191.0).collect::<Vec<_>>(),
    )
    .unwrap();
    let cscale = ColorScale::new([0.0, 1.0])
        .unwrap()
        .color_map(ColorMap::Turbo);

    let hollow_pts = PlotData::new(vec![[0.0, 0.0], [10.0, 10.0]]);
    let center_marker_pt = PlotData::new(vec![[5.0, 5.0]]);
    let g1: Vec<f64> = (0..32).map(|i| 10.0 + (i as f64) * 1.5).collect();
    let g2: Vec<f64> = (0..32).map(|i| 25.0 + (i as f64) * 1.8).collect();
    let box_d = BoxChartData::from_groups(&[1.0, 2.0], &[&g1, &g2], 0.5).unwrap();
    let violin_d =
        ViolinData::from_groups(&[1.0, 2.0], &[&g1, &g2], ViolinOptions::default()).unwrap();
    let rain_d =
        RaincloudData::from_groups(&[1.0, 2.0], &[&g1, &g2], ViolinOptions::default()).unwrap();

    let ctx = Context::default();
    let mut states = [
        PlotState::default(),
        PlotState::default(),
        PlotState::default(),
        PlotState::default(),
    ];
    states[0].set_brushed_bounds(Some(
        PlotBounds::new([t0 + 1200.0, 1.2], [t0 + 4200.0, 2.9]).unwrap(),
    ));
    let _ = &PointerButton::Primary;
    let _ = Modifiers::ALT;
    let _ = Event::PointerMoved(Pos2::ZERO);

    let mut hollow_rect = Rect::NOTHING;
    let mut snapshot = None;
    ctx.run_ui(input(vec![]), |ui| {
        snapshot = Some(
            PlotSnapshot::capture(ui, Default::default(), |ui| {
                let fig = Figure::new("export_stage12")
                    .columns(2)
                    .cell_height(260.0)
                    .show(ui, 4, |cell| match cell.index() {
                        0 => cell.show_layers(
                            Plot::new("dt_brush")
                                .x_axis(Axis::new().datetime())
                                .allow_brush(true)
                                .title("DateTime + Brush"),
                            &mut states[0],
                            &[PlotLayer::from(
                                Line::new(&dt_pts).color(Color32::LIGHT_BLUE),
                            )],
                        ),
                        1 => cell.show_layers(
                            Plot::new("turbo_map").title("ColorMap::Turbo"),
                            &mut states[1],
                            &[PlotLayer::from(Heatmap::new(&grid, cscale))],
                        ),
                        2 => {
                            let r = cell.show_layers(
                                Plot::new("hollow_markers")
                                    .background(Color32::WHITE)
                                    .show_grid(false)
                                    .title("Hollow + Star"),
                                &mut states[2],
                                &[
                                    PlotLayer::from(
                                        Line::new(&hollow_pts)
                                            .width(0.0)
                                            .markers(MarkerStyle::new(MarkerShape::Pentagram, 9.0))
                                            .color(Color32::BLUE),
                                    ),
                                    PlotLayer::from(
                                        Scatter::new(&center_marker_pt)
                                            .shape(
                                                MarkerStyle::hollow(MarkerShape::Circle, 16.0)
                                                    .shape,
                                            )
                                            .radius(16.0)
                                            .color(Color32::RED),
                                    ),
                                ],
                            );
                            hollow_rect = r.transform.rect();
                            r
                        }
                        _ => cell.show_layers(
                            Plot::new("h_dist").title("Horizontal Distributions"),
                            &mut states[3],
                            &[
                                PlotLayer::from(
                                    BoxChart::new(&box_d).orientation(BarOrientation::Horizontal),
                                ),
                                PlotLayer::from(
                                    ViolinPlot::new(&violin_d)
                                        .orientation(BarOrientation::Horizontal),
                                ),
                                PlotLayer::from(
                                    Raincloud::new(&rain_d).orientation(BarOrientation::Horizontal),
                                ),
                            ],
                        ),
                    })
                    .unwrap();
                fig.response.rect
            })
            .unwrap(),
        );
    })
    .drop_without_applying_deltas();
    drop((
        dt_pts,
        grid,
        hollow_pts,
        center_marker_pt,
        box_d,
        violin_d,
        rain_d,
        states,
        ctx,
    ));

    let opts = ExportOptions {
        pixels_per_point: 1.0,
        ..Default::default()
    };
    let snap = snapshot.unwrap();
    let svg = String::from_utf8(snap.to_svg(opts).unwrap()).unwrap();
    assert!(roxmltree::Document::parse(&svg).is_ok());
    assert!(svg.contains("<svg") && svg.contains("clipPath"));
    assert!(!svg.contains("<text"));

    let png_bytes = snap.to_png(opts).unwrap();
    assert!(png_bytes.starts_with(b"\x89PNG"));
    let decoder = png::Decoder::new(std::io::Cursor::new(&png_bytes));
    let mut reader = decoder.read_info().unwrap();
    let mut buf = vec![0; reader.output_buffer_size().unwrap()];
    let info = reader.next_frame(&mut buf).unwrap();
    let bytes = &buf[..info.buffer_size()];
    // Center of cell 2 at data [5.0, 5.0] must stay white (hollow center, NOT solid red).
    let cx = hollow_rect.center().x.round() as u32;
    let cy = hollow_rect.center().y.round() as u32;
    let stride = info.width as usize * 4;
    let idx = cy as usize * stride + cx as usize * 4;
    let center_rgb = &bytes[idx..idx + 3];
    assert_eq!(
        center_rgb,
        &[255, 255, 255],
        "expected white hollow center, got {center_rgb:?}"
    );
}
