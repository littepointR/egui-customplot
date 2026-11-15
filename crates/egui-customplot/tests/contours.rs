mod support;

use egui::{Color32, Context, Event, Stroke};
use egui_customplot::{
    ColorBar, ColorMap, ColorScale, ContourData, ContourLevels, Contours, FilledContourData,
    FilledContours, FunctionContourError, Plot, PlotBounds, PlotState, RegionKind, ScalarField2D,
};
use support::{capture, run};

fn saddle_field() -> ScalarField2D {
    let bounds = PlotBounds::new([-2.0, -2.0], [2.0, 2.0]).unwrap();
    ScalarField2D::sample_function(bounds, [25, 25], 10_000, |x, y| x * x - y * y).unwrap()
}

#[test]
fn scalar_field_validates_budget_and_contours_extract_isolines_and_labels() {
    let bounds = PlotBounds::new([-1.0, -1.0], [1.0, 1.0]).unwrap();
    assert!(matches!(
        ContourData::from_function(
            bounds,
            [32, 32],
            ContourLevels::Count(5),
            100,
            |x, y| x + y
        ),
        Err(err) if matches!(err.to_string().as_str(), s if s.contains("exceeding budget"))
    ));
    assert!(matches!(
        ScalarField2D::sample_function(bounds, [10, 10], 50, |x, y| x + y),
        Err(FunctionContourError::BudgetExceeded {
            required: 100,
            max_evaluations: 50
        })
    ));

    let field = saddle_field();
    let contours = ContourData::from_field(&field, ContourLevels::Count(6)).unwrap();
    assert_eq!(contours.levels().len(), 6);
    assert!(!contours.lines().is_empty());

    let ctx = Context::default();
    let mut state = PlotState::default();
    let scale = ColorScale::new([-4.0, 4.0])
        .unwrap()
        .color_map(ColorMap::BlueRed);
    let (out, output) = capture(&ctx, vec![], false, |ui| {
        Plot::new("contours").show_layers(
            ui,
            &mut state,
            &[Contours::new(&contours)
                .color_scale(scale)
                .show_labels(true)
                .width(1.5)
                .into()],
        )
    });
    assert_eq!(out.diagnostics.contours.input_lines, contours.lines().len());
    assert!(out.diagnostics.contours.output_paths > 0);
    assert!(out.diagnostics.contours.output_labels > 0);
    output.drop_without_applying_deltas();

    let pt = contours.lines()[0].points[0];
    let pos = out.transform.position_from_point(pt);
    let hit = run(&ctx, vec![Event::PointerMoved(pos)], false, |ui| {
        Plot::new("contours").show_layers(
            ui,
            &mut state,
            &[Contours::new(&contours).color_scale(scale).into()],
        )
    });
    assert!(hit.hovered_point.is_some());
}

#[test]
fn filled_contours_render_bands_borders_colorbar_and_capture_hover() {
    let field = saddle_field();
    let levels = ContourLevels::Explicit(vec![-3.0, -1.0, 0.0, 1.0, 3.0]);
    let bands = FilledContourData::from_field(&field, levels.clone()).unwrap();
    let lines = ContourData::from_field(&field, levels).unwrap();
    let scale = ColorScale::new([-3.0, 3.0])
        .unwrap()
        .color_map(ColorMap::Viridis);

    let ctx = Context::default();
    let mut state = PlotState::default();
    let out = run(&ctx, vec![], false, |ui| {
        let resp = Plot::new("contourf").show_layers(
            ui,
            &mut state,
            &[FilledContours::new(&bands, scale)
                .border_lines(&lines, Stroke::new(1.0, Color32::BLACK))
                .into()],
        );
        ColorBar::new(scale).show(ui);
        resp
    });
    assert_eq!(out.diagnostics.contours.input_bands, 5);
    assert!(out.diagnostics.contours.output_meshes >= 1);
    assert!(out.diagnostics.contours.output_paths > 0);

    // At (1.5, 0.0), z = 2.25 in [1.0, 3.0].
    let pos = out.transform.position_from_point([1.5, 0.0]);
    let hovered = run(&ctx, vec![Event::PointerMoved(pos)], false, |ui| {
        Plot::new("contourf").show_layers(
            ui,
            &mut state,
            &[FilledContours::new(&bands, scale).into()],
        )
    });
    let region = hovered.hovered_region.unwrap();
    assert_eq!(region.kind, RegionKind::FilledContour);
    assert_eq!(region.contour_band, Some([1.0, 3.0]));
}

#[test]
fn ezcontour_and_ezcontourf_use_default_two_pi_domain() {
    let lines =
        ContourData::ezcontour(ContourLevels::Count(5), 10_000, |x, y| x.sin() * y.cos()).unwrap();
    let bands =
        FilledContourData::ezcontourf(ContourLevels::Count(5), 10_000, |x, y| x.sin() * y.cos())
            .unwrap();
    assert!(!lines.lines().is_empty());
    assert_eq!(bands.bands().len(), 6);

    let ctx = Context::default();
    let scale = ColorScale::new([-1.0, 1.0]).unwrap();
    let mut state = PlotState::default();
    let out = run(&ctx, vec![], false, |ui| {
        Plot::new("ez").show_layers(
            ui,
            &mut state,
            &[
                FilledContours::new(&bands, scale).into(),
                Contours::new(&lines).color(Color32::WHITE).into(),
            ],
        )
    });
    let b = out.transform.bounds();
    assert!(b.min()[0] <= -6.0 && b.max()[0] >= 6.0);
    assert!(b.min()[1] <= -6.0 && b.max()[1] >= 6.0);
}
