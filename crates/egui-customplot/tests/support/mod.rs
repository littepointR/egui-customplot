use egui::{Context, Event, RawInput, Rect, Ui};
use egui_customplot::PlotResponse;

pub fn input(events: Vec<Event>) -> RawInput {
    RawInput {
        screen_rect: Some(Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::vec2(800.0, 600.0),
        )),
        events,
        ..Default::default()
    }
}

pub fn run(
    ctx: &Context,
    events: Vec<Event>,
    multipass: bool,
    mut show: impl FnMut(&mut Ui) -> PlotResponse,
) -> PlotResponse {
    let mut result = None;
    let mut passes = 0;
    ctx.run_ui(input(events), |ui| {
        result = Some(show(ui));
        if multipass && passes == 0 {
            ui.ctx().request_discard("test final layout");
        }
        passes += 1;
    })
    .drop_without_applying_deltas();
    if multipass {
        assert_eq!(passes, 2);
    }
    result.unwrap()
}
