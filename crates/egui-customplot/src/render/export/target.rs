use egui::{Context, Id, Pos2, RawInput, Rect, Ui, Vec2, ViewportId};

use super::{CaptureLimits, ExportError, PlotSnapshot};

#[derive(Clone, Copy)]
struct RasterDensity(f32);

pub(crate) fn export_density(ctx: &Context) -> Option<f32> {
    ctx.data(|data| data.get_temp::<RasterDensity>(Id::NULL).map(|v| v.0))
}

struct DensityScope<'a>(&'a Context);
impl Drop for DensityScope<'_> {
    fn drop(&mut self) {
        self.0
            .data_mut(|data| data.remove::<RasterDensity>(Id::NULL));
    }
}

impl PlotSnapshot {
    /// Render a fresh layout in a caller-owned, idle export context. Configure its fonts
    /// and theme as desired. Call outside an active UI pass, using separate plot state.
    /// Canvas dimensions are logical points. Return the desired region of the canvas
    /// from the callback. egui may repeat it for layout, so keep the callback replayable.
    /// Pass the same density to `ExportOptions` when encoding. Fonts are rasterized
    /// at this density and software depth buffers use up to 2048 pixels per edge.
    pub fn render_at(
        ctx: &Context,
        canvas: Vec2,
        pixels_per_point: f32,
        limits: CaptureLimits,
        mut draw: impl FnMut(&mut Ui) -> Rect,
    ) -> Result<Self, ExportError> {
        let area = Rect::from_min_size(Pos2::ZERO, canvas);
        if !area.is_finite()
            || !area.is_positive()
            || !pixels_per_point.is_finite()
            || pixels_per_point <= 0.0
            || pixels_per_point > 16.0
            || (canvas * pixels_per_point).max_elem() > 32768.0
        {
            return Err(ExportError::InvalidDimensions);
        }
        let mut input = RawInput {
            screen_rect: Some(area),
            ..Default::default()
        };
        input
            .viewports
            .get_mut(&ViewportId::ROOT)
            .unwrap()
            .native_pixels_per_point = Some(pixels_per_point / ctx.zoom_factor());
        let mut result = Err(ExportError::InvalidGeometry);
        ctx.run_ui(input, |ui| {
            ctx.data_mut(|data| data.insert_temp(Id::NULL, RasterDensity(pixels_per_point)));
            let _scope = DensityScope(ctx);
            result = Self::capture(ui, limits, |ui| draw(ui).intersect(area));
        })
        .drop_without_applying_deltas();
        result
    }
}
