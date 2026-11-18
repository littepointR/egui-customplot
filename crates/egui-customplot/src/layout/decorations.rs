use crate::ColorScale;
use egui::{Color32, Rect, Ui};
use std::sync::Arc;

/// Built-in plot content handed to a replacement title or color bar.
#[derive(Clone, Copy)]
pub enum DecorationContent<'a> {
    Title(&'a str),
    ColorBar { scale: ColorScale, label: &'a str },
}

/// Measure a title's height or a right-hand color bar's width in logical points.
/// Painting happens in a clipped child UI outside the navigation rectangle.
pub trait PlotDecoration: Send + Sync {
    fn thickness(&self, ui: &Ui, content: DecorationContent<'_>) -> f32;
    fn show(&self, ui: &mut Ui, area: Rect, content: DecorationContent<'_>, text_color: Color32);
}

#[derive(Default)]
pub(crate) struct DecorationSlots {
    pub title: Option<Arc<dyn PlotDecoration>>,
    pub color_bar: Option<Arc<dyn PlotDecoration>>,
}

pub(crate) struct PlacedDecoration<'a> {
    component: &'a dyn PlotDecoration,
    content: DecorationContent<'a>,
    area: Rect,
}

impl DecorationSlots {
    pub fn reserve<'a>(
        &'a self,
        ui: &Ui,
        outer: &mut Rect,
        title: &'a str,
        bar: Option<&'a (ColorScale, String)>,
    ) -> [Option<PlacedDecoration<'a>>; 2] {
        let mut place = |component: &'a dyn PlotDecoration, content| {
            let mut area = *outer;
            let size = component.thickness(ui, content);
            if !size.is_finite() || size <= 0. {
                return None;
            }
            match content {
                DecorationContent::Title(_) => {
                    area.max.y = area.min.y + size.min(area.height() * 0.4);
                    outer.min.y = area.bottom();
                }
                DecorationContent::ColorBar { .. } => {
                    area.min.x = area.max.x - size.min(area.width() * 0.4);
                    outer.max.x = area.left();
                }
            }
            Some(PlacedDecoration {
                component,
                content,
                area,
            })
        };
        [
            self.title
                .as_deref()
                .and_then(|c| place(c, DecorationContent::Title(title))),
            self.color_bar
                .as_deref()
                .zip(bar)
                .and_then(|(c, (scale, label))| {
                    place(
                        c,
                        DecorationContent::ColorBar {
                            scale: *scale,
                            label,
                        },
                    )
                }),
        ]
    }
}

pub(crate) fn show_decorations(
    ui: &mut Ui,
    id: egui::Id,
    placed: [Option<PlacedDecoration<'_>>; 2],
    color: Color32,
) {
    for (i, entry) in placed.into_iter().enumerate() {
        if let Some(p) = entry
            && ui.is_rect_visible(p.area)
        {
            let mut child = ui.new_child(
                egui::UiBuilder::new()
                    .id_salt(id.with(("decoration", i)))
                    .max_rect(p.area),
            );
            child.set_clip_rect(p.area.intersect(ui.clip_rect()));
            p.component.show(&mut child, p.area, p.content, color);
        }
    }
}
