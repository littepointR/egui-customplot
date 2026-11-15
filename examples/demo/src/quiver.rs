//! Host-owned vector records; rendering never rescales or reconstructs the data.
use eframe::egui;
use egui_customplot::{
    Axis, AxisScale, Figure, Line, LinePattern, Plot, PlotData, PlotResponse, PlotState, Quiver,
    QuiverArrow, QuiverData, YAxis,
};

pub(super) struct QuiverDemo {
    pub enabled: bool,
    pub heads: bool,
    pub dashed: bool,
    pub reversed: bool,
    pub head_length: f32,
    field: QuiverData,
    positive: QuiverData,
    reference: PlotData,
    states: [PlotState; 2],
}

impl Default for QuiverDemo {
    fn default() -> Self {
        let mut arrows = Vec::new();
        for y in -3..=3 {
            for x in -5..=5 {
                let (x, y) = (f64::from(x), f64::from(y));
                arrows.push(QuiverArrow::new([x, y], [-0.18 * y, 0.18 * x]));
            }
        }
        arrows.push(QuiverArrow::new([f64::NAN, 1.0], [1.0; 2]));
        Self {
            enabled: false,
            heads: true,
            dashed: false,
            reversed: false,
            head_length: 10.0,
            field: QuiverData::new(arrows),
            positive: QuiverData::new(vec![
                QuiverArrow::new([1.0, 10.0], [1.0, 30.0]),
                QuiverArrow::new([3.0, 50.0], [1.0, -40.0]),
                QuiverArrow::new([5.0, 20.0], [1.0, 80.0]),
                QuiverArrow::new([7.0, 80.0], [1.0, -30.0]),
            ]),
            reference: PlotData::new(vec![[0.0, -1.0], [9.0, 1.0]]),
            states: Default::default(),
        }
    }
}

impl QuiverDemo {
    #[cfg(test)]
    pub fn sample_identities(&self) -> [usize; 2] {
        [
            self.field.arrows().as_ptr() as usize,
            self.positive.arrows().as_ptr() as usize,
        ]
    }

    pub fn controls(&mut self, ui: &mut egui::Ui) {
        ui.separator();
        ui.checkbox(&mut self.enabled, "Explicit 2D vector field / quiver");
        ui.add_enabled_ui(self.enabled, |ui| {
            ui.checkbox(&mut self.heads, "True-endpoint arrowheads");
            ui.checkbox(&mut self.dashed, "Dashed shafts / solid heads");
            ui.checkbox(&mut self.reversed, "Reverse X");
            ui.add(egui::Slider::new(&mut self.head_length, 2.0..=24.0).text("Head logical points"));
            if ui.button("Fit vector panels").clicked() {
                self.states.iter_mut().for_each(PlotState::reset);
            }
            ui.small("Origin + displacement, no implicit autoscale. Hover original origins/tips, not clipped intersections or shafts. Zero vectors are not drawn. Exact scan; no vector LOD/index/cache.");
        });
    }

    pub fn show(&mut self, ui: &mut egui::Ui) -> Vec<PlotResponse> {
        let height = ((ui.available_height() - 110.0) * 0.5).max(80.0);
        let pattern = if self.dashed {
            LinePattern::Dashed {
                length: 8.0,
                gap: 4.0,
            }
        } else {
            LinePattern::Solid
        };
        Figure::new("vector-panels")
            .cell_height(height)
            .show(ui, 2, |cell| {
                let index = cell.index();
                let plot = Plot::new("plot")
                    .height(height)
                    .allow_scroll_zoom(true)
                    .legend(true)
                    .x_axis(Axis::new().reversed(self.reversed));
                let quiver = |data| {
                    Quiver::new(data)
                        .show_heads(self.heads)
                        .head_size(self.head_length, self.head_length * 0.8)
                        .pattern(pattern)
                };
                if index == 0 {
                    cell.show_layers(
                        plot.title("Rotational field: explicit data-space vectors")
                            .data_aspect(1.0),
                        &mut self.states[0],
                        &[quiver(&self.field).name("Rotation / no autoscale").into()],
                    )
                } else {
                    cell.show_layers(
                        plot.title("Straight transformed-endpoint arrows on right log Y")
                            .right_y_axis(
                                Axis::new().scale(AxisScale::Log10).label("Amplitude (log)"),
                            ),
                        &mut self.states[1],
                        &[
                            Line::new(&self.reference).name("Left reference").into(),
                            quiver(&self.positive)
                                .name("Right vectors")
                                .y_axis(YAxis::Right)
                                .color(egui::Color32::LIGHT_RED)
                                .into(),
                        ],
                    )
                }
            })
            .unwrap()
            .inner
    }
}
