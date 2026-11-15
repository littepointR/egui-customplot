//! Host-owned scalar field contours, pcolor shading, and RGBA images.
use eframe::egui;
use egui_customplot::{
    ColorBar, ColorMap, ColorScale, ContourData, ContourLevels, Contours, Figure,
    FilledContourData, FilledContours, ImageData, Pcolor, PcolorShading, Plot, PlotBounds,
    PlotImage, PlotResponse, PlotState, ScalarField2D,
};

pub(super) struct ContourDemo {
    pub enabled: bool,
    pub show_pcolor: bool,
    pub interp_shading: bool,
    pub show_labels: bool,
    pub colors: ColorMap,
    field: ScalarField2D,
    lines: ContourData,
    filled: FilledContourData,
    image: ImageData,
    states: [PlotState; 2],
}

impl Default for ContourDemo {
    fn default() -> Self {
        let bounds = PlotBounds::new([-3.0, -3.0], [3.0, 3.0]).expect("valid bounds");
        let field = ScalarField2D::sample_function(bounds, [40, 36], 10_000, |x, y| {
            x.sin() * y.cos() + 0.35 * (-(x * x + y * y) * 0.25).exp()
        })
        .expect("within evaluation budget");
        let levels = ContourLevels::Count(8);
        let lines = ContourData::from_field(&field, levels.clone()).expect("valid levels");
        let filled = FilledContourData::from_field(&field, levels).expect("valid levels");

        let (w, h) = (32usize, 24usize);
        let mut rgba = Vec::with_capacity(w * h * 4);
        for r in 0..h {
            for c in 0..w {
                if (c < 4 && r < 4) || (c >= w - 4 && r >= h - 4) {
                    rgba.extend_from_slice(&[0, 0, 0, 0]);
                } else {
                    let fx = c as f32 / (w - 1) as f32;
                    let fy = r as f32 / (h - 1) as f32;
                    let checker = if ((c / 4) + (r / 4)) % 2 == 0 {
                        220.0
                    } else {
                        70.0
                    };
                    let red = (fx * checker) as u8;
                    let green = (fy * 210.0) as u8;
                    let blue = ((1.0 - fx) * 200.0) as u8;
                    rgba.extend_from_slice(&[red, green, blue, 255]);
                }
            }
        }
        let image = ImageData::rgba8(
            w,
            h,
            PlotBounds::new([0.0, 0.0], [w as f64, h as f64]).expect("valid image bounds"),
            rgba,
        )
        .expect("matching RGBA length");

        Self {
            enabled: false,
            show_pcolor: false,
            interp_shading: false,
            show_labels: true,
            colors: ColorMap::Viridis,
            field,
            lines,
            filled,
            image,
            states: Default::default(),
        }
    }
}

impl ContourDemo {
    #[cfg(test)]
    pub fn sample_identities(&self) -> [usize; 2] {
        [
            self.field.values().as_ptr() as usize,
            self.image.rgba_bytes().as_ptr() as usize,
        ]
    }

    pub fn controls(&mut self, ui: &mut egui::Ui) {
        ui.separator();
        ui.checkbox(&mut self.enabled, "Contours, pcolor & images");
        ui.add_enabled_ui(self.enabled, |ui| {
            egui::ComboBox::from_id_salt("contour-color-map")
                .selected_text(format!("{:?}", self.colors))
                .show_ui(ui, |ui| {
                    for map in [ColorMap::Viridis, ColorMap::Gray, ColorMap::BlueRed] {
                        ui.selectable_value(&mut self.colors, map, format!("{map:?}"));
                    }
                });
            ui.checkbox(&mut self.show_pcolor, "Pcolor instead of contourf");
            ui.add_enabled(
                self.show_pcolor,
                egui::Checkbox::new(&mut self.interp_shading, "Interpolated pcolor shading"),
            );
            ui.checkbox(&mut self.show_labels, "Contour level labels");
            if ui.button("Fit contour/image panels").clicked() {
                self.states.iter_mut().for_each(PlotState::reset);
            }
            ui.small("Marching Squares contours extracted once on host; imshow_axes reverses Y with 1:1 aspect.");
        });
    }

    pub fn show(&mut self, ui: &mut egui::Ui) -> Vec<PlotResponse> {
        let height = ((ui.available_height() - 120.0) * 0.5).max(80.0);
        let total_h = (ui.available_height() - 100.0).max(120.0);
        let scale = ColorScale::new([-1.0, 1.35])
            .expect("valid range")
            .color_map(self.colors);
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
            ColorBar::new(scale)
                .label("Scalar z")
                .height(total_h)
                .show(ui);
            Figure::new("contour-panels")
                .cell_height(height)
                .show(ui, 2, |cell| {
                    if cell.index() == 0 {
                        let plot = Plot::new("contour-plot")
                            .height(height)
                            .allow_scroll_zoom(true)
                            .legend(true);
                        if self.show_pcolor {
                            let shading = if self.interp_shading {
                                PcolorShading::Interp
                            } else {
                                PcolorShading::Flat
                            };
                            cell.show_layers(
                                plot.title("Pcolor Rectilinear Field + Contour Isolines"),
                                &mut self.states[0],
                                &[
                                    Pcolor::new(&self.field, scale)
                                        .name("Pcolor")
                                        .shading(shading)
                                        .into(),
                                    Contours::new(&self.lines)
                                        .name("Isolines")
                                        .color(egui::Color32::WHITE)
                                        .show_labels(self.show_labels)
                                        .into(),
                                ],
                            )
                        } else {
                            cell.show_layers(
                                plot.title("Filled Contours (contourf) + Labeled Isolines"),
                                &mut self.states[0],
                                &[
                                    FilledContours::new(&self.filled, scale)
                                        .name("Filled contours")
                                        .into(),
                                    Contours::new(&self.lines)
                                        .name("Isolines")
                                        .color(egui::Color32::from_gray(235))
                                        .show_labels(self.show_labels)
                                        .into(),
                                ],
                            )
                        }
                    } else {
                        cell.show_layers(
                            Plot::new("imshow-plot")
                                .imshow_axes()
                                .height(height)
                                .allow_scroll_zoom(true)
                                .legend(true)
                                .title("RGBA Image (imshow_axes: top-down Y, 1:1 aspect)"),
                            &mut self.states[1],
                            &[PlotImage::new(&self.image).name("RGBA image").into()],
                        )
                    }
                })
                .unwrap()
                .inner
        })
        .inner
    }
}
