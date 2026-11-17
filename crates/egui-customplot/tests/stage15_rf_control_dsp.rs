use std::f64::consts::TAU;

use egui::{Context, Pos2, RawInput, Rect};
use egui_customplot::{
    AxisScale, BodePlot, BodePlotState, ColorMap, ColorScale, CwtData, CwtPlot,
    FrequencyResponseData, NicholsPlot, NyquistPlot, Plot, PlotLayer, PlotState, PoleZeroData,
    PspectrumData, RootLocus, RootLocusData, SmithGridKind, SmithPlot, SmithPlotData,
    SmithPlotState, SmithTransform, SpectrogramData, TransferFunction, WindowKind, ZPlanePlot,
    fft_arbitrary, fft_radix2, fft_real,
};

fn raw() -> RawInput {
    RawInput {
        screen_rect: Some(Rect::from_min_size(Pos2::ZERO, egui::vec2(720.0, 520.0))),
        ..Default::default()
    }
}

fn hovered_raw(pos: Pos2) -> RawInput {
    let mut input = raw();
    input.events.push(egui::Event::PointerMoved(pos));
    input
}

#[test]
fn ext19_smith_conformal_roundtrip_vswr_and_hover_pick() {
    assert_eq!(SmithTransform::z_to_gamma([1.0, 0.0]), Some([0.0, 0.0]));
    assert!((SmithTransform::vswr_from_gamma([1.0 / 3.0, 0.0]) - 2.0).abs() < 1e-12);
    assert!((SmithTransform::return_loss_db([0.5, 0.0]) - 6.020_599_913_28).abs() < 1e-6);

    for [r, x] in [
        [0.0, 0.5],
        [0.2, -1.5],
        [1.0, 1.0],
        [3.5, -0.4],
        [10.0, 4.0],
    ] {
        let g = SmithTransform::z_to_gamma([r, x]).unwrap();
        assert!(
            g[0].hypot(g[1]) <= 1.0 + 1e-12,
            "passive z must map inside unit disk"
        );
        let z_back = SmithTransform::gamma_to_z(g).unwrap();
        assert!((z_back[0] - r).abs() < 1e-10 && (z_back[1] - x).abs() < 1e-10);
    }

    let freqs = [1e9, 2e9, 3e9];
    let z = [[25.0, -25.0], [50.0, 0.0], [100.0, 50.0]];
    let smith = SmithPlotData::from_impedance(&freqs, &z, 50.0)
        .unwrap()
        .with_vswr_circles(&[2.0]);

    let ctx = Context::default();
    let mut state = SmithPlotState::default();
    let mut p1 = Pos2::ZERO;
    ctx.run_ui(raw(), |ui| {
        let resp = SmithPlot::new("s", &smith)
            .grid_kind(SmithGridKind::ImpedanceAdmittance)
            .show(ui, &mut state);
        p1 = resp.transform.gamma_to_pos(smith.traces()[0].gamma[1]);
    })
    .drop_without_applying_deltas();

    ctx.run_ui(hovered_raw(p1), |ui| {
        let resp = SmithPlot::new("s", &smith).show(ui, &mut state);
        let hit = resp.hovered_point.expect("smith hover");
        assert_eq!(hit.point_index, 1);
        let zo = hit.z_ohms.unwrap();
        assert!((zo[0] - 50.0).abs() < 1e-8 && zo[1].abs() < 1e-8);
        assert!((hit.vswr - 1.0).abs() < 1e-8);
    })
    .drop_without_applying_deltas();
}

#[test]
fn ext20_zplane_transfer_function_multiplicity_and_stability() {
    // H(z) = (1 - 2 z^-1 + z^-2) / (1 - z^-1 + 0.25 z^-2) -> double zero at +1, double pole at +0.5
    let pz = PoleZeroData::from_transfer_fn(&[1.0, -2.0, 1.0], &[1.0, -1.0, 0.25]).unwrap();
    assert!(pz.is_causal_stable());
    assert_eq!(pz.zeros().len(), 1);
    assert_eq!(pz.zeros()[0].multiplicity, 2);
    assert!((pz.zeros()[0].re - 1.0).abs() < 1e-4);
    assert_eq!(pz.poles().len(), 1);
    assert_eq!(pz.poles()[0].multiplicity, 2);
    assert!((pz.poles()[0].re - 0.5).abs() < 1e-4);

    let unstable = PoleZeroData::from_zeros_poles(&[], &[[1.1, 0.0]]).unwrap();
    assert!(!unstable.is_causal_stable());

    let ctx = Context::default();
    let mut state = PlotState::default();
    let mut p_pos = Pos2::ZERO;
    ctx.run_ui(raw(), |ui| {
        let resp = Plot::new("zp").zplane_axes().show_layers(
            ui,
            &mut state,
            &[PlotLayer::from(ZPlanePlot::new(&pz))],
        );
        p_pos = resp.transform.position_from_point([0.5, 0.0]);
    })
    .drop_without_applying_deltas();

    ctx.run_ui(hovered_raw(p_pos), |ui| {
        let resp = Plot::new("zp").zplane_axes().show_layers(
            ui,
            &mut state,
            &[PlotLayer::from(ZPlanePlot::new(&pz))],
        );
        assert!(resp.hovered_point.is_some());
    })
    .drop_without_applying_deltas();
}

#[test]
fn ext21_control_bode_margins_nyquist_nichols_and_rlocus() {
    // G(s) = 2 / (s + 1)^3 = 2 / (s^3 + 3 s^2 + 3 s + 1)
    // Exact analytical values: w_pc = sqrt(3) ~ 1.73205, GM = 20*log10(4) ~ 12.0412 dB
    let plant = TransferFunction::new(&[2.0], &[1.0, 3.0, 3.0, 1.0]).unwrap();
    let fr = FrequencyResponseData::from_transfer_fn(&plant, 0.05, 30.0, 400).unwrap();
    let margins = fr.margins();
    let w_pc = margins.phase_crossover_rad_s.expect("w_pc");
    let gm_db = margins.gain_margin_db.expect("gm_db");
    assert!((w_pc - 3.0_f64.sqrt()).abs() < 0.05, "w_pc={w_pc}");
    assert!((gm_db - 12.0412).abs() < 0.25, "gm_db={gm_db}");
    assert!(margins.phase_margin_deg.expect("pm_deg") > 30.0);

    // Unwrapped phase drops monotonically to ~ -270 deg at high omega
    assert!(fr.phase_curve().points().last().unwrap()[1] < -230.0);

    let rl = RootLocusData::auto_gains(&plant, 20.0, 80).unwrap();
    assert_eq!(rl.open_loop_poles().len(), 3);
    assert_eq!(rl.branches().len(), 3);

    let ctx = Context::default();
    let mut bode_state = BodePlotState::default();
    let mut nyq_state = PlotState::default();
    let mut nich_state = PlotState::default();
    let mut rl_state = PlotState::default();
    let mut passes = 0;
    ctx.run_ui(raw(), |ui| {
        let br = BodePlot::new("bode", &fr)
            .height(240.0)
            .show(ui, &mut bode_state);
        assert!(br.mag.transform.bounds().min()[0] > 0.0);
        if passes == 0 {
            ui.ctx().request_discard("stage15 multipass");
        }
        passes += 1;
    })
    .drop_without_applying_deltas();
    assert_eq!(passes, 2);
    assert_eq!(bode_state.mag.bounds(), bode_state.phase.bounds());

    let mut nq_pos = Pos2::ZERO;
    ctx.run_ui(raw(), |ui| {
        let r = Plot::new("nyq").data_aspect(1.0).show_layers(
            ui,
            &mut nyq_state,
            &[PlotLayer::from(NyquistPlot::new(&fr))],
        );
        nq_pos = r
            .transform
            .position_from_point(fr.nyquist_curve().points()[120]);
    })
    .drop_without_applying_deltas();
    ctx.run_ui(hovered_raw(nq_pos), |ui| {
        let r = Plot::new("nyq").data_aspect(1.0).show_layers(
            ui,
            &mut nyq_state,
            &[PlotLayer::from(NyquistPlot::new(&fr))],
        );
        assert_eq!(r.hovered_point.unwrap().point_index, 120);
    })
    .drop_without_applying_deltas();

    ctx.run_ui(raw(), |ui| {
        let r1 = Plot::new("nich").show_layers(
            ui,
            &mut nich_state,
            &[PlotLayer::from(NicholsPlot::new(&fr))],
        );
        assert!(r1.outer_rect.is_positive());
        let r2 =
            Plot::new("rl").show_layers(ui, &mut rl_state, &[PlotLayer::from(RootLocus::new(&rl))]);
        assert!(r2.outer_rect.is_positive());
    })
    .drop_without_applying_deltas();
}

#[test]
fn ext22_pure_rust_fft_bluestein_welch_stft_and_cwt_coi() {
    // Parseval identity for Radix-2 (64) and non-power-of-two Bluestein (37)
    for n in [64usize, 37usize] {
        let sig: Vec<f64> = (0..n)
            .map(|k| (0.23 * k as f64).sin() + 0.5 * (0.71 * k as f64).cos())
            .collect();
        let mut c: Vec<[f64; 2]> = sig.iter().map(|&x| [x, 0.0]).collect();
        let spec = if n.is_power_of_two() {
            fft_radix2(&mut c, false);
            c
        } else {
            fft_arbitrary(&c, false)
        };
        let energy_time: f64 = sig.iter().map(|x| x * x).sum();
        let energy_freq: f64 = spec.iter().map(|[r, i]| r * r + i * i).sum::<f64>() / n as f64;
        assert!((energy_time - energy_freq).abs() < 1e-9, "n={n}");

        let inv = fft_arbitrary(&spec, true);
        for (x, r) in sig.iter().zip(inv.iter()) {
            assert!((x - r[0]).abs() < 1e-9 && r[1].abs() < 1e-9);
        }
        assert_eq!(fft_real(&sig).len(), n);
    }

    // Welch peak localization on exact 24 Hz + 64 Hz tones at fs = 256 Hz
    let fs = 256.0;
    let sig: Vec<f64> = (0..512)
        .map(|i| {
            let t = i as f64 / fs;
            2.0 * (TAU * 24.0 * t).sin() + (TAU * 64.0 * t).cos()
        })
        .collect();
    let welch = PspectrumData::welch(&sig, fs, 128, 64, WindowKind::Hann).unwrap();
    let peak = welch
        .psd_db_curve()
        .points()
        .iter()
        .max_by(|a, b| a[1].total_cmp(&b[1]))
        .unwrap();
    assert!(
        (peak[0] - 24.0).abs() <= 2.5,
        "expected peak near 24 Hz, got {}",
        peak[0]
    );

    let spec = SpectrogramData::stft(&sig, fs, 64, 32, 128, WindowKind::Hamming).unwrap();
    assert!(spec.grid().columns() > 5 && spec.grid().rows() == 65);

    let cwt = CwtData::morlet_logspace(&sig[..256], fs, 8.0, 96.0, 20).unwrap();
    assert_eq!(cwt.coi_left().len(), 20);
    assert_eq!(cwt.coi_right().len(), 20);

    let scale = ColorScale::new(cwt.value_range())
        .unwrap()
        .color_map(ColorMap::Viridis);
    let ctx = Context::default();
    let mut st = PlotState::default();
    ctx.run_ui(raw(), |ui| {
        let r = Plot::new("cwt")
            .x_axis(egui_customplot::Axis::new().scale(AxisScale::Linear))
            .show_layers(ui, &mut st, &[PlotLayer::from(CwtPlot::new(&cwt, scale))]);
        assert!(r.outer_rect.is_positive());
    })
    .drop_without_applying_deltas();
}
