use egui::{Pos2, Rect, Vec2};

pub(crate) const MAX_FIGURE_CELLS: usize = 256;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FigureError {
    TooManyCells { requested: usize, maximum: usize },
    NonFiniteLayout,
}

impl std::fmt::Display for FigureError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TooManyCells { requested, maximum } => write!(
                f,
                "figure requested {requested} cells; maximum is {maximum}"
            ),
            Self::NonFiniteLayout => {
                f.write_str("figure layout cannot be represented by finite coordinates")
            }
        }
    }
}

impl std::error::Error for FigureError {}

pub(crate) struct FigureLayout {
    pub size: Vec2,
    pub columns: usize,
    cell_size: Vec2,
    spacing: Vec2,
}

impl FigureLayout {
    pub fn new(
        width: f32,
        columns: usize,
        count: usize,
        cell_height: f32,
        spacing: Vec2,
    ) -> Result<Self, FigureError> {
        if count > MAX_FIGURE_CELLS {
            return Err(FigureError::TooManyCells {
                requested: count,
                maximum: MAX_FIGURE_CELLS,
            });
        }
        let columns = columns.clamp(1, 64).min(count.max(1));
        let rows = count.div_ceil(columns);
        // Keep a positive cell even when the parent becomes narrower than all
        // gaps combined. Parent and per-cell clipping remain authoritative.
        let gap_x = spacing.x.min(width / (2 * columns) as f32);
        let cell_width = (width - gap_x * columns.saturating_sub(1) as f32) / columns as f32;
        let height = rows as f32 * cell_height + rows.saturating_sub(1) as f32 * spacing.y;
        let size = egui::vec2(width, height);
        if !size.is_finite() || width <= 0.0 || height < 0.0 || cell_width <= 0.0 {
            return Err(FigureError::NonFiniteLayout);
        }
        Ok(Self {
            size,
            columns,
            cell_size: egui::vec2(cell_width, cell_height),
            spacing: egui::vec2(gap_x, spacing.y),
        })
    }

    pub fn cell_rect(&self, origin: Pos2, index: usize) -> Result<Rect, FigureError> {
        let offset = egui::vec2((index % self.columns) as f32, (index / self.columns) as f32)
            * (self.cell_size + self.spacing);
        let rect = Rect::from_min_size(origin + offset, self.cell_size);
        if !rect.is_finite() || !rect.is_positive() {
            return Err(FigureError::NonFiniteLayout);
        }
        Ok(rect)
    }
}
