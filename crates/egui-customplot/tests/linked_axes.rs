mod support;

use egui::{Context, Event, Modifiers};
use egui_customplot::{
    Axis, AxisLink, AxisScale, Figure, Line, LinkError, LinkReport, Plot, PlotBounds, PlotData,
    PlotResponse, PlotState,
};
use support::run;

fn show(
    ctx: &Context,
    states: &mut [PlotState; 2],
    data: [&PlotData; 2],
    link: &mut AxisLink,
    events: Vec<Event>,
    scale: AxisScale,
    changing_height: bool,
) -> (Vec<PlotResponse>, LinkReport) {
    let mut result = None;
    let mut passes = 0;
    ctx.run_ui(support::input(events), |ui| {
        let height = if changing_height && passes == 0 {
            180.0
        } else {
            240.0
        };
        let responses = Figure::new("figure")
            .columns(2)
            .cell_height(height)
            .show(ui, 2, |cell| {
                let i = cell.index();
                cell.show_plot(
                    Plot::new("plot")
                        .allow_scroll_zoom(true)
                        .x_axis(Axis::new().scale(scale).reversed(i == 1))
                        .y_axis(Axis::new().scale(scale)),
                    &mut states[i],
                    &[Line::new(data[i])],
                )
            })
            .unwrap()
            .inner;
        let [a, b] = states;
        let report = link.synchronize(ui.ctx(), &mut [a, b]).unwrap();
        result = Some((responses, report));
        if changing_height && passes == 0 {
            ui.ctx().request_discard("final figure height");
        }
        passes += 1;
    })
    .drop_without_applying_deltas();
    result.unwrap()
}

fn wheel(pos: egui::Pos2) -> Vec<Event> {
    vec![
        Event::PointerMoved(pos),
        Event::MouseWheel {
            unit: egui::MouseWheelUnit::Point,
            delta: egui::vec2(0.0, 40.0),
            phase: egui::TouchPhase::Move,
            source: egui::MouseWheelSource::Unknown,
            modifiers: Modifiers::NONE,
        },
    ]
}

fn range(response: &PlotResponse, axis: usize) -> [f64; 2] {
    [
        response.transform.bounds().min()[axis],
        response.transform.bounds().max()[axis],
    ]
}

#[test]
fn auto_union_follows_data_and_unlinked_axes_remain_independent() {
    let ctx = Context::default();
    let a = PlotData::new(vec![[1.0, 1.0], [2.0, 2.0]]);
    let b = PlotData::new(vec![[10.0, 100.0], [20.0, 200.0]]);
    let mut states = Default::default();
    let mut link = AxisLink::new([true, false]);
    let (out, report) = show(
        &ctx,
        &mut states,
        [&a, &b],
        &mut link,
        vec![],
        AxisScale::Linear,
        false,
    );
    assert_eq!(range(&out[0], 0), range(&out[1], 0));
    assert!(range(&out[0], 0)[0] < 1.0 && range(&out[0], 0)[1] > 20.0);
    assert_ne!(range(&out[0], 1), range(&out[1], 1));
    assert_eq!(
        report.changed_plots, 0,
        "settled second pass must not repaint indefinitely"
    );
    assert!(states.iter().all(|s| s.bounds().is_none()));
    let b = PlotData::new(vec![[100.0, 1000.0], [200.0, 2000.0]]);
    let (out, _) = show(
        &ctx,
        &mut states,
        [&a, &b],
        &mut link,
        vec![],
        AxisScale::Linear,
        false,
    );
    assert!(range(&out[1], 0)[1] > 200.0);
    assert!(range(&out[1], 1)[1] > 2000.0);
    assert_eq!(range(&out[0], 0), range(&out[1], 0));
    states[0].clear_axis_links();
    let single = run(&ctx, vec![], false, |ui| {
        Plot::new("single").show(ui, &mut states[0], &[Line::new(&a)])
    });
    assert!(range(&single, 0)[1] < 3.0);
}

#[test]
fn navigation_from_either_submission_order_shares_data_ranges_without_double_zoom() {
    for scale in [AxisScale::Linear, AxisScale::Log10] {
        for source in 0..2 {
            let mut outcomes = Vec::new();
            for changing_height in [false, true] {
                let ctx = Context::default();
                ctx.options_mut(|o| o.max_passes = std::num::NonZeroUsize::new(4).unwrap());
                let a = PlotData::new(vec![[1.0, 1.0], [100.0, 100.0]]);
                let mut states = Default::default();
                let mut link = AxisLink::new([true, true]);
                let (initial, _) =
                    show(&ctx, &mut states, [&a, &a], &mut link, vec![], scale, false);
                let pos = initial[source].transform.rect().center() + egui::vec2(10.0, -10.0);
                show(
                    &ctx,
                    &mut states,
                    [&a, &a],
                    &mut link,
                    vec![Event::PointerMoved(pos)],
                    scale,
                    false,
                );
                let (out, report) = show(
                    &ctx,
                    &mut states,
                    [&a, &a],
                    &mut link,
                    wheel(pos),
                    scale,
                    changing_height,
                );
                assert_eq!(report.sources, [Some(source); 2]);
                assert_eq!(out[0].transform.bounds(), out[1].transform.bounds());
                assert_eq!(report.conflicting_actions, 0);
                assert!(
                    out[source].transform.bounds().span()[0]
                        < initial[source].transform.bounds().span()[0]
                );
                // Horizontal geometry is unchanged by the deliberate final-pass
                // height change: X must be exactly equal, never compounded.
                outcomes.push(range(&out[source], 0));
            }
            assert_eq!(outcomes[0], outcomes[1]);
        }
    }
}

#[test]
fn home_resets_the_group_to_auto_union_not_just_the_active_member() {
    let ctx = Context::default();
    let a = PlotData::new(vec![[1.0; 2], [2.0; 2]]);
    let b = PlotData::new(vec![[10.0; 2], [20.0; 2]]);
    let mut states = Default::default();
    let mut link = AxisLink::new([true, false]);
    link.set_bounds(PlotBounds::new([5.0, -1.0], [8.0, 1.0]).unwrap());
    let (initial, _) = show(
        &ctx,
        &mut states,
        [&a, &b],
        &mut link,
        vec![],
        AxisScale::Linear,
        false,
    );
    assert_eq!(range(&initial[0], 0), [5.0, 8.0]);
    ctx.memory_mut(|m| m.request_focus(initial[1].response.id));
    let key = Event::Key {
        key: egui::Key::Home,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: Modifiers::NONE,
    };
    let (out, report) = show(
        &ctx,
        &mut states,
        [&a, &b],
        &mut link,
        vec![key],
        AxisScale::Linear,
        false,
    );
    assert_eq!(report.sources[0], Some(1));
    assert_eq!(range(&out[0], 0), range(&out[1], 0));
    assert!(range(&out[0], 0)[0] < 1.0 && range(&out[0], 0)[1] > 20.0);
}

#[test]
fn validation_is_transactional_for_scales_aspect_domains_context_and_stale_views() {
    let ctx = Context::default();
    let mut states: [PlotState; 2] = Default::default();
    let mut link = AxisLink::new([true, false]);
    assert_eq!(
        link.synchronize(&ctx, &mut [&mut states[0]]),
        Err(LinkError::NotShown { member: 0 })
    );
    for mode in ["scale", "aspect", "domain", "context"] {
        ctx.run_ui(support::input(vec![]), |ui| {
            for (i, state) in states.iter_mut().enumerate() {
                let mut plot = Plot::new(i).height(100.0);
                if (mode == "scale" && i == 1) || mode == "domain" {
                    plot = plot.x_axis(Axis::new().scale(AxisScale::Log10));
                }
                if mode == "aspect" && i == 1 {
                    plot = plot.data_aspect(1.0);
                }
                plot.show(ui, state, &[]);
            }
            let [a, b] = &mut states;
            let error = match mode {
                "scale" => LinkError::ScaleMismatch { member: 1, axis: 0 },
                "aspect" => LinkError::AspectConstraint { member: 1 },
                "domain" => {
                    link.set_bounds(PlotBounds::new([-2.0; 2], [-1.0; 2]).unwrap());
                    LinkError::InvalidRange { axis: 0 }
                }
                _ => LinkError::StaleView { member: 0 },
            };
            let other = Context::default();
            assert_eq!(
                link.synchronize(
                    if mode == "context" { &other } else { ui.ctx() },
                    &mut [a, b]
                ),
                Err(error)
            );
        })
        .drop_without_applying_deltas();
        link.reset();
    }
    let [a, b] = &mut states;
    assert_eq!(
        link.synchronize(&ctx, &mut [a, b]),
        Err(LinkError::StaleView { member: 0 })
    );
    assert!(states.iter().all(|s| s.bounds().is_none()));
}
