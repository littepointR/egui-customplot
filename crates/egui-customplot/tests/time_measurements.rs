use egui::{Context, Event, Modifiers, PointerButton, Pos2};
use egui_customplot::{
    Line, Plot, PlotData, PlotState, SharedCursors, StepResponseData, StepResponseError,
    TransferFunction,
};
#[path = "support/input.rs"]
mod support;

#[test]
fn analytic_steps_cover_poles_feedthrough_and_sampled_settling() {
    let cases: &[(&[f64], &[f64], f64)] = &[
        (&[1.0], &[1.0, 1.0], 1.0 - (-1.0_f64).exp()),
        (&[-2.0], &[1.0, 1.0], -2.0 * (1.0 - (-1.0_f64).exp())),
        (&[1.0], &[1.0, 2.0, 1.0], 1.0 - 2.0 * (-1.0_f64).exp()),
        (&[2.0], &[1.0, 3.0, 2.0], (1.0 - (-1.0_f64).exp()).powi(2)),
        (
            &[1.0, 2.0, 2.0],
            &[1.0, 2.0, 1.0],
            2.0 - 2.0 * (-1.0_f64).exp(),
        ),
    ];
    for &(num, den, expected) in cases {
        let tf = TransferFunction::new(num, den).unwrap();
        let step = StepResponseData::new(&tf, 10.0, 1001).unwrap();
        assert!((step.curve().points()[100][1] - expected).abs() < 1e-12);
    }
    let tf = TransferFunction::new(&[1.0], &[1.0, 1.0]).unwrap();
    let step = StepResponseData::new(&tf, 5.0, 501).unwrap();
    assert_eq!(step.settling_time(), Some(3.92));
    assert_eq!(step.overshoot_percent(), Some(0.0));
    assert_eq!(
        StepResponseData::new(&tf, 1.0, 10).unwrap().settling_time(),
        None
    );
    let under = TransferFunction::new(&[2.0], &[1.0, 2.0, 2.0]).unwrap();
    let step = StepResponseData::new(&under, 2.0 * std::f64::consts::PI, 101).unwrap();
    assert!(
        (step.overshoot_percent().unwrap() - 100.0 * (-std::f64::consts::PI).exp()).abs() < 1e-10
    );
    assert_eq!(
        StepResponseData::new(&under.with_delay(1.0).unwrap(), 1.0, 2).unwrap_err(),
        StepResponseError::UnsupportedSystem
    );
}

#[test]
fn pins_survive_final_layout_and_shared_x_stays_in_data_coordinates() {
    let ctx = Context::default();
    let data = PlotData::new(vec![[0., 0.], [1., 1.]]);
    let mut states = [PlotState::default(), PlotState::default()];
    let mut cursors = SharedCursors::default();
    let mut frame = |events: Vec<Event>, mods: Modifiers, retry: bool| {
        let mut input = support::input(events);
        input.events.insert(0, Event::ModifiersChanged(mods));
        let mut pass = 0;
        let mut tf = None;
        ctx.run_ui(input, |ui| {
            let top = Plot::new("time")
                .height(if retry && pass == 0 { 125. } else { 180. })
                .show(ui, &mut states[0], &[Line::new(&data)]);
            let bottom =
                Plot::new("frequency")
                    .height(160.)
                    .show(ui, &mut states[1], &[Line::new(&data)]);
            tf = Some((top.transform, bottom.transform));
            cursors.show(ui, &[&top, &bottom]);
            if retry && pass == 0 {
                ui.ctx().request_discard("final layout");
            }
            pass += 1;
        })
        .drop_without_applying_deltas();
        (tf.unwrap(), cursors.measurements())
    };
    let ((t, _), _) = frame(vec![], Modifiers::NONE, false);
    let mut click = |pos: Pos2, mods: Modifiers| {
        frame(
            vec![
                Event::PointerMoved(pos),
                Event::PointerButton {
                    pos,
                    button: PointerButton::Primary,
                    pressed: true,
                    modifiers: mods,
                },
            ],
            mods,
            false,
        );
        frame(
            vec![Event::PointerButton {
                pos,
                button: PointerButton::Primary,
                pressed: false,
                modifiers: mods,
            }],
            mods,
            true,
        )
    };
    let pa = t.position_from_point([0.3, 0.6]);
    let ((t, bottom), a) = click(pa, Modifiers::NONE);
    assert_eq!(a.a.unwrap().point, t.point_from_position(pa));
    let pb = t.position_from_point([0.7, 0.5]);
    let (_, b) = click(pb, Modifiers::SHIFT);
    assert!((b.delta_x().unwrap() - 0.4).abs() < 1e-5);
    assert!((b.delta_y().unwrap() + 0.1).abs() < 1e-5);
    assert_eq!(a.a, b.a);
    let (_, cross) = click(bottom.position_from_point([0.6, 0.8]), Modifiers::SHIFT);
    assert_eq!(cross.a, a.a);
    assert_ne!(cross.a.unwrap().plot, cross.b.unwrap().plot);
    assert!(cross.delta_x().is_some());
    assert_eq!(cross.delta_y(), None);
    assert_eq!(cursors.measurements(), cross);
}
