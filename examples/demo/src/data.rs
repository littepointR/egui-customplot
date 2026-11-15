use eframe::egui::Color32;
use egui_customplot::PlotData;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) enum Scene {
    #[default]
    Signal,
    PowerLaw,
}

pub(super) struct DemoData {
    pub signal: PlotData,
    pub reference: PlotData,
    pub secondary: PlotData,
    pub observations: PlotData,
    pub colors: Vec<Color32>,
    pub radii: Vec<f32>,
}

impl DemoData {
    pub fn new(scene: Scene, count: usize, gaps: bool) -> Self {
        let count = count.max(2);
        let x_at = |t: f64| match scene {
            Scene::Signal => 20.0 * t,
            Scene::PowerLaw => 10.0_f64.powf(-2.0 + 5.0 * t),
        };
        let reference = |x: f64| match scene {
            Scene::Signal => x.sin(),
            Scene::PowerLaw => x.powf(0.65),
        };
        let signal = PlotData::new(
            (0..count)
                .map(|i| {
                    let t = i as f64 / (count - 1) as f64;
                    let x = x_at(t);
                    let y = if gaps && (0.4..0.435).contains(&t) {
                        f64::NAN
                    } else {
                        match scene {
                            Scene::Signal => reference(x) + 0.12 * (13.0 * x).sin(),
                            Scene::PowerLaw => reference(x) * (1.0 + 0.12 * (x.ln() * 6.0).sin()),
                        }
                    };
                    [x, y]
                })
                .collect::<Vec<_>>(),
        );
        let reference = PlotData::new(
            (0..256)
                .map(|i| {
                    let x = x_at(i as f64 / 255.0);
                    [x, reference(x)]
                })
                .collect::<Vec<_>>(),
        );
        let observations = PlotData::new(
            (0..80)
                .map(|i| {
                    let x = x_at((i as f64 + 0.5) / 80.0);
                    let perturbation = ((i * 31 % 17) as f64 / 16.0 - 0.5) * 0.4;
                    let y = match scene {
                        Scene::Signal => x.sin() + perturbation,
                        Scene::PowerLaw => x.powf(0.65) * (1.0 + perturbation),
                    };
                    [x, y]
                })
                .collect::<Vec<_>>(),
        );
        let secondary = PlotData::new(
            (0..256)
                .map(|i| {
                    let t = i as f64 / 255.0;
                    [x_at(t), 1000.0 + 500.0 * (t * 10.0).cos()]
                })
                .collect::<Vec<_>>(),
        );
        let colors = (0..80)
            .map(|i| Color32::from_rgb(220, 90 + (i * 23 % 140) as u8, 65))
            .collect();
        let radii = (0..80).map(|i| 3.0 + (i % 5) as f32).collect();
        Self {
            signal,
            reference,
            secondary,
            observations,
            colors,
            radii,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui_customplot::AxisScale;

    #[test]
    fn power_law_data_is_log_valid_and_styles_match_samples() {
        let data = DemoData::new(Scene::PowerLaw, 1000, false);
        for samples in [&data.signal, &data.reference, &data.observations] {
            assert_eq!(samples.invalid_points([AxisScale::Log10; 2]), 0);
        }
        assert_eq!(data.colors.len(), data.observations.len());
        assert_eq!(data.radii.len(), data.observations.len());
    }

    #[test]
    fn gaps_are_explicit_and_tiny_requests_are_safe() {
        for scene in [Scene::Signal, Scene::PowerLaw] {
            assert!(DemoData::new(scene, 1000, true).signal.non_finite_points() > 0);
            assert_eq!(DemoData::new(scene, 0, false).signal.len(), 2);
        }
    }
}
