use crate::{AxisScale, AxisTick, ColorScale, PlotBounds, PlotTransform};
use egui::{Mesh, Rect, Ui};

pub(crate) fn paint_color_bar(
    ui: &Ui,
    outer: Rect,
    bar: Rect,
    scale: ColorScale,
    ticks: &[AxisTick],
    label: &str,
    text_color: egui::Color32,
) {
    let painter = ui.painter().with_clip_rect(outer);
    let mut mesh = Mesh::default();
    for i in 0..64 {
        let fraction = f64::from(i) / 64.0;
        let rect = Rect::from_min_max(
            egui::pos2(
                bar.left(),
                bar.bottom() - bar.height() * (i + 1) as f32 / 64.0,
            ),
            egui::pos2(bar.right(), bar.bottom() - bar.height() * i as f32 / 64.0),
        );
        mesh.add_colored_rect(rect, scale.map().sample(fraction + 0.5 / 64.0));
    }
    painter.add(egui::Shape::mesh(mesh));
    let range = scale.range();
    let transform = PlotTransform::with_axes(
        PlotBounds::new([0.0, range[0]], [1.0, range[1]]).expect("color scale"),
        bar,
        [AxisScale::Linear, scale.scale()],
        [false; 2],
    );
    let font = egui::TextStyle::Small.resolve(ui.style());
    if let Some(transform) = transform {
        for tick in ticks.iter().filter(|t| t.major) {
            let pos = transform.position_from_point([1.0, tick.value]) + egui::vec2(4.0, 0.0);
            painter.text(
                pos,
                egui::Align2::LEFT_CENTER,
                &tick.label,
                font.clone(),
                text_color,
            );
        }
    }
    painter.rect_stroke(
        bar,
        0.0,
        ui.visuals().widgets.noninteractive.bg_stroke,
        egui::StrokeKind::Inside,
    );
    painter.text(
        outer.left_top(),
        egui::Align2::LEFT_TOP,
        label,
        font,
        text_color,
    );
}
