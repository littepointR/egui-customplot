mod cell;

use std::sync::Arc;

use egui::{AsIdSalt, IdSalt, Response, Ui, UiBuilder, Vec2};

use crate::{Annotation, FigureError, PlotStyle, PlotTheme, layout::FigureLayout};
pub use cell::FigureCell;

/// An immediate-mode grid container. Samples and each PlotState stay with the
/// host; cells inherit explicit defaults without changing the host Ui style.
/// This is not a window, event loop or a persistent MATLAB handle graph.
pub struct Figure {
    id_salt: IdSalt,
    columns: usize,
    cell_height: f32,
    spacing: Vec2,
    style: PlotStyle,
    theme: Option<Arc<PlotTheme>>,
    annotations: Arc<[Annotation]>,
}

pub struct FigureResponse<R> {
    pub response: Response,
    pub inner: Vec<R>,
    pub cell_rects: Vec<egui::Rect>,
    pub output_annotations: usize,
    pub annotation_rejections: usize,
}

impl Figure {
    pub fn new(id_salt: impl AsIdSalt) -> Self {
        Self {
            id_salt: IdSalt::new(id_salt),
            columns: 2,
            cell_height: 260.0,
            spacing: egui::vec2(8.0, 8.0),
            style: PlotStyle::default(),
            theme: None,
            annotations: Arc::from([]),
        }
    }

    /// Clamped to 1..=64; the final row can be incomplete. No plots are dropped.
    pub fn columns(mut self, columns: usize) -> Self {
        self.columns = columns.clamp(1, 64);
        self
    }

    pub fn cell_height(mut self, height: f32) -> Self {
        if height.is_finite() && height > 0.0 {
            self.cell_height = height.max(16.0);
        }
        self
    }

    pub fn spacing(mut self, spacing: Vec2) -> Self {
        if spacing.is_finite() && spacing.x >= 0.0 && spacing.y >= 0.0 {
            self.spacing = spacing;
        }
        self
    }

    pub fn style(mut self, style: PlotStyle) -> Self {
        self.style = style.over(self.style);
        self
    }

    /// A Plot's own theme, if specified, takes precedence over this default.
    pub fn theme(mut self, theme: impl Into<Arc<PlotTheme>>) -> Self {
        self.theme = Some(theme.into());
        self
    }

    /// Figure-level normalized `[0, 1]^2` `EX-027` annotations spanning all cells.
    pub fn annotations(mut self, annotations: impl Into<Arc<[Annotation]>>) -> Self {
        self.annotations = annotations.into();
        self
    }

    /// Run the callback for each of at most 256 cells, in row-major order.
    /// Like egui UI closures, it can run again in discarded layout passes.
    pub fn show<R>(
        self,
        ui: &mut Ui,
        cell_count: usize,
        mut show: impl FnMut(&mut FigureCell<'_>) -> R,
    ) -> Result<FigureResponse<R>, FigureError> {
        let width = if ui.available_width().is_finite() {
            ui.available_width().max(1.0)
        } else {
            ui.clip_rect()
                .intersect(ui.ctx().viewport_rect())
                .width()
                .max(1.0)
        };
        let layout = FigureLayout::new(
            width,
            self.columns,
            cell_count,
            self.cell_height,
            self.spacing,
        )?;
        // The allocated origin accounts for bottom-up, right-to-left and
        // centered/justified parents; next_widget_position is only a cursor.
        let (rect, response) = ui.allocate_exact_size(layout.size, egui::Sense::hover());
        // Validate every cell before invoking any user callback.
        let cell_rects = (0..cell_count)
            .map(|i| layout.cell_rect(rect.min, i))
            .collect::<Result<Vec<_>, _>>()?;
        let mut inner = Vec::with_capacity(cell_count);
        for (index, rect) in cell_rects.iter().copied().enumerate() {
            let mut child = ui.new_child(
                UiBuilder::new()
                    .id_salt((self.id_salt, index))
                    .max_rect(rect)
                    .layout(egui::Layout::top_down(egui::Align::Min)),
            );
            child.set_clip_rect(ui.clip_rect().intersect(rect));
            let mut cell = FigureCell::new(
                &mut child,
                index,
                layout.columns,
                rect,
                self.style,
                self.theme.as_ref(),
            );
            inner.push(show(&mut cell));
        }
        let (output_annotations, annotation_rejections) = if self.annotations.is_empty() {
            (0, 0)
        } else {
            let style = crate::model::resolve_plot_style(
                ui,
                self.theme.as_deref(),
                PlotStyle::default(),
                self.style,
            );
            crate::render::paint_figure_annotations(ui, &self.annotations, rect, &style)
        };
        Ok(FigureResponse {
            response,
            inner,
            cell_rects,
            output_annotations,
            annotation_rejections,
        })
    }
}
