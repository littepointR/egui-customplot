use eframe::egui;
use egui_customplot::{Axis, AxisScale, MarkerShape, Plot};

use crate::data::Scene;

pub(super) struct Options {
    pub scene: Scene,
    pub count: usize,
    pub gaps: bool,
    pub second_view: bool,
    pub grid: bool,
    pub legend: bool,
    pub hover: bool,
    pub markers: bool,
    pub dashed: bool,
    pub equal_units: bool,
    pub observations: bool,
    pub log_axes: [bool; 2],
    pub reversed: [bool; 2],
    pub lock_y: bool,
    pub width: f32,
    pub radius: f32,
    pub shape: MarkerShape,
    pub color: egui::Color32,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            scene: Scene::Signal,
            count: 10_000,
            gaps: true,
            second_view: false,
            grid: true,
            legend: true,
            hover: true,
            markers: false,
            dashed: false,
            equal_units: false,
            observations: true,
            log_axes: [false; 2],
            reversed: [false; 2],
            lock_y: false,
            width: 1.5,
            radius: 3.5,
            shape: MarkerShape::Circle,
            color: egui::Color32::from_rgb(70, 160, 235),
        }
    }
}

pub(super) struct Changes {
    pub rebuild: bool,
    pub reset: bool,
}

impl Options {
    pub fn plot(&self, id: &'static str, height: f32) -> Plot {
        let axis = |index: usize, label: &str| {
            Axis::new()
                .scale(if self.log_axes[index] {
                    AxisScale::Log10
                } else {
                    AxisScale::Linear
                })
                .reversed(self.reversed[index])
                .minor_grid(true)
                .label(label)
        };
        let plot = Plot::new(id)
            .height(height)
            .show_grid(self.grid)
            .allow_scroll_zoom(true)
            .drag_axes([true, !self.lock_y])
            .zoom_axes([true, !self.lock_y])
            .title(match self.scene {
                Scene::Signal => "Signal and observations",
                Scene::PowerLaw => "Power law: linear / semilog / loglog",
            })
            .x_axis(axis(0, "x"))
            .y_axis(axis(1, "Amplitude"))
            .legend(self.legend)
            .show_hover(self.hover);
        if self.equal_units {
            plot.data_aspect(1.0)
        } else {
            plot
        }
    }
}

pub(super) fn show(ui: &mut egui::Ui, options: &mut Options) -> Changes {
    let old_data = (options.scene, options.count, options.gaps);
    let old_axes = (options.log_axes, options.reversed);
    ui.heading("Plot controls");
    egui::widgets::global_theme_preference_buttons(ui);
    ui.separator();
    ui.selectable_value(&mut options.scene, Scene::Signal, "Signal + scatter");
    ui.selectable_value(&mut options.scene, Scene::PowerLaw, "Positive power law");
    if options.scene != old_data.0 {
        options.log_axes = [options.scene == Scene::PowerLaw; 2];
    }
    egui::ComboBox::from_id_salt("samples")
        .selected_text(format!("{} points", options.count))
        .show_ui(ui, |ui| {
            for count in [1_000, 10_000, 100_000, 1_000_000] {
                ui.selectable_value(&mut options.count, count, format!("{count} points"));
            }
        });
    ui.checkbox(&mut options.gaps, "NaN gap");
    ui.checkbox(&mut options.second_view, "Independent second view");
    ui.separator();
    ui.checkbox(&mut options.grid, "Grid (major + log minor)");
    ui.checkbox(&mut options.legend, "Clickable legend");
    ui.checkbox(&mut options.hover, "Raw-point hover + crosshair");
    ui.checkbox(
        &mut options.observations,
        "Per-point colored / sized scatter",
    );
    ui.checkbox(&mut options.markers, "Markers on every line sample");
    ui.checkbox(&mut options.dashed, "Dashed signal stroke");
    ui.checkbox(&mut options.equal_units, "Equal X/Y units (log: decades)");
    if options.markers && options.count >= 100_000 {
        ui.colored_label(
            egui::Color32::YELLOW,
            "Large marker meshes: no LOD / budget yet.",
        );
    }
    ui.horizontal(|ui| {
        ui.checkbox(&mut options.log_axes[0], "Log X");
        ui.checkbox(&mut options.log_axes[1], "Log Y");
    });
    ui.horizontal(|ui| {
        ui.checkbox(&mut options.reversed[0], "Reverse X");
        ui.checkbox(&mut options.reversed[1], "Reverse Y");
    });
    ui.checkbox(&mut options.lock_y, "Lock Y navigation");
    ui.color_edit_button_srgba(&mut options.color);
    ui.add(egui::Slider::new(&mut options.width, 0.0..=5.0).text("Line width"));
    ui.add(egui::Slider::new(&mut options.radius, 1.0..=10.0).text("Line marker radius"));
    egui::ComboBox::from_id_salt("shape")
        .selected_text(format!("{:?}", options.shape))
        .show_ui(ui, |ui| {
            for shape in [
                MarkerShape::Circle,
                MarkerShape::Square,
                MarkerShape::Diamond,
                MarkerShape::TriangleUp,
                MarkerShape::TriangleDown,
                MarkerShape::Plus,
                MarkerShape::Cross,
            ] {
                ui.selectable_value(&mut options.shape, shape, format!("{shape:?}"));
            }
        });
    let reset = ui.button("Auto-fit both views").clicked();
    ui.separator();
    ui.small("Drag: pan\nWheel: zoom\nDouble-click / focused Home: fit\nLegend: toggle visibility");
    ui.small("Nonpositive log samples are omitted / break lines. Raw data is never changed.");
    Changes {
        rebuild: old_data != (options.scene, options.count, options.gaps),
        reset: reset
            || old_axes != (options.log_axes, options.reversed)
            || old_data.0 != options.scene,
    }
}
