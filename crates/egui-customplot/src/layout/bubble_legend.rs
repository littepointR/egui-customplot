use egui::{Align2, Color32, CornerRadius, Rect, Response, Sense, Stroke, StrokeKind, Ui, Vec2};

/// Native egui legend widget displaying data-size-to-radius correspondence for bubble plots.
#[derive(Clone, Debug)]
pub struct BubbleLegend {
    title: String,
    value_range: [f64; 2],
    radius_range: [f32; 2],
    steps: usize,
    explicit: Vec<(f64, f32)>,
    color: Option<Color32>,
    stroke: Option<Stroke>,
}

/// Response returned by [`BubbleLegend::show`].
pub struct BubbleLegendResponse {
    pub response: Response,
    pub rect: Rect,
    pub samples: Vec<(f64, f32)>,
}

impl BubbleLegend {
    pub fn new(value_range: [f64; 2], radius_range: [f32; 2]) -> Self {
        let lo = radius_range[0].clamp(1.0, 64.0);
        let hi = radius_range[1].clamp(lo, 64.0);
        Self {
            title: String::new(),
            value_range,
            radius_range: [lo, hi],
            steps: 3,
            explicit: Vec::new(),
            color: None,
            stroke: None,
        }
    }

    pub fn from_entries(entries: impl IntoIterator<Item = (f64, f32)>) -> Self {
        let explicit: Vec<(f64, f32)> = entries
            .into_iter()
            .filter(|(v, r)| v.is_finite() && r.is_finite() && *r > 0.0)
            .take(6)
            .map(|(v, r)| (v, r.clamp(1.0, 64.0)))
            .collect();
        let mut legend = Self::new([0.0, 1.0], [4.0, 16.0]);
        legend.explicit = explicit;
        legend
    }

    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    pub fn steps(mut self, steps: usize) -> Self {
        self.steps = steps.clamp(2, 6);
        self
    }

    pub fn color(mut self, color: Color32) -> Self {
        self.color = Some(color);
        self
    }

    pub fn stroke(mut self, stroke: Stroke) -> Self {
        self.stroke = Some(stroke);
        self
    }

    pub(crate) fn resolved_samples(&self) -> Vec<(f64, f32)> {
        if !self.explicit.is_empty() {
            return self.explicit.clone();
        }
        let [v0, v1] = self.value_range;
        if !v0.is_finite() || !v1.is_finite() || v1 < v0 {
            return Vec::new();
        }
        if (v1 - v0).abs() <= f64::EPSILON {
            return vec![(v0, 0.5 * (self.radius_range[0] + self.radius_range[1]))];
        }
        let count = self.steps.clamp(2, 6);
        (0..count)
            .map(|idx| {
                let t = idx as f64 / (count - 1) as f64;
                let value = v0 + (v1 - v0) * t;
                let radius = bubble_pixel_radius(value, self.value_range, self.radius_range);
                (value, radius)
            })
            .collect()
    }

    pub fn show(self, ui: &mut Ui) -> BubbleLegendResponse {
        let samples = self.resolved_samples();
        let desired = bubble_legend_size(&self, &samples);
        let (rect, response) = ui.allocate_exact_size(desired, Sense::hover());
        if ui.is_rect_visible(rect) {
            paint_bubble_legend(ui, rect, &self, &samples, ui.visuals().text_color());
        }
        BubbleLegendResponse {
            response,
            rect,
            samples,
        }
    }
}

/// Computes bubble pixel radius using proportional area scaling across `[v_min, v_max]`.
pub(crate) fn bubble_pixel_radius(
    value: f64,
    value_range: [f64; 2],
    radius_range: [f32; 2],
) -> f32 {
    let r_min = radius_range[0].clamp(1.0, 128.0);
    let r_max = radius_range[1].clamp(r_min, 128.0);
    let span = value_range[1] - value_range[0];
    if !value.is_finite() || !span.is_finite() || span <= 0.0 {
        return 0.5 * (r_min + r_max);
    }
    let t = ((value - value_range[0]) / span).clamp(0.0, 1.0) as f32;
    let a0 = r_min * r_min;
    let a1 = r_max * r_max;
    (a0 + (a1 - a0) * t).sqrt()
}

pub(crate) fn bubble_legend_size(legend: &BubbleLegend, samples: &[(f64, f32)]) -> Vec2 {
    let max_r = samples.iter().map(|(_, r)| *r).fold(4.0_f32, f32::max);
    let header = if legend.title.is_empty() { 8.0 } else { 24.0 };
    let rows_h: f32 = samples.iter().map(|(_, r)| (r * 2.0).max(16.0) + 6.0).sum();
    let width = (max_r * 2.0 + 78.0).max(if legend.title.is_empty() {
        84.0
    } else {
        (legend.title.len() as f32 * 7.5 + 24.0).min(240.0)
    });
    egui::vec2(width, (header + rows_h + 8.0).max(48.0))
}

pub(crate) fn paint_bubble_legend(
    ui: &Ui,
    rect: Rect,
    legend: &BubbleLegend,
    samples: &[(f64, f32)],
    text_color: Color32,
) {
    let painter = ui.painter().with_clip_rect(rect);
    let bg = ui
        .visuals()
        .widgets
        .noninteractive
        .bg_fill
        .gamma_multiply(0.92);
    painter.rect_filled(rect, CornerRadius::same(4), bg);
    painter.rect_stroke(
        rect,
        CornerRadius::same(4),
        ui.visuals().widgets.noninteractive.bg_stroke,
        StrokeKind::Inside,
    );
    let font = egui::TextStyle::Small.resolve(ui.style());
    let mut y = rect.top() + 8.0;
    if !legend.title.is_empty() {
        painter.text(
            egui::pos2(rect.left() + 8.0, y),
            Align2::LEFT_TOP,
            &legend.title,
            font.clone(),
            text_color,
        );
        y += 18.0;
    }
    let max_r = samples.iter().map(|(_, r)| *r).fold(4.0_f32, f32::max);
    let cx = rect.left() + 8.0 + max_r;
    let label_x = rect.left() + 8.0 + max_r * 2.0 + 8.0;
    let fill = legend
        .color
        .unwrap_or_else(|| Color32::from_rgba_unmultiplied(66, 135, 245, 96));
    let stroke = legend
        .stroke
        .unwrap_or_else(|| Stroke::new(1.2, text_color.gamma_multiply(0.75)));
    for &(value, radius) in samples {
        let row_h = (radius * 2.0).max(16.0);
        let cy = y + row_h * 0.5;
        painter.circle(egui::pos2(cx, cy), radius, fill, stroke);
        painter.text(
            egui::pos2(label_x, cy),
            Align2::LEFT_CENTER,
            format_legend_value(value),
            font.clone(),
            text_color,
        );
        y += row_h + 6.0;
    }
}

fn format_legend_value(v: f64) -> String {
    let abs = v.abs();
    if abs >= 1e5 || (abs > 0.0 && abs < 1e-3) {
        format!("{v:.2e}")
    } else if (v - v.round()).abs() < 1e-6 {
        format!("{v:.0}")
    } else {
        format!("{v:.2}")
    }
}
