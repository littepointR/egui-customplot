use egui::{Color32, Stroke};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PolarRegionKind {
    Theta,
    Radius,
    Polar,
}

/// Polar region annotation (`thetaregion` / `radiusregion` / `polarregion`, `EX-033`..`EX-035`).
#[derive(Clone, Debug, PartialEq)]
pub struct PolarRegionAnnotation {
    kind: PolarRegionKind,
    theta_range: Option<[f64; 2]>,
    r_range: Option<[f64; 2]>,
    fill: Color32,
    border: Option<Stroke>,
    label: String,
}

impl PolarRegionAnnotation {
    /// Angular sector band spanning `theta_start..=theta_end` (supports wrap-around when `end < start`).
    pub fn theta_region(theta_start_rad: f64, theta_end_rad: f64) -> Self {
        Self {
            kind: PolarRegionKind::Theta,
            theta_range: Some([theta_start_rad, theta_end_rad]),
            r_range: None,
            fill: Color32::from_rgba_unmultiplied(70, 130, 180, 48),
            border: None,
            label: String::new(),
        }
    }

    /// Circular disk / annular band spanning `[r_inner, r_outer]`.
    pub fn radius_region(r_inner: f64, r_outer: f64) -> Self {
        Self {
            kind: PolarRegionKind::Radius,
            theta_range: None,
            r_range: Some([r_inner, r_outer]),
            fill: Color32::from_rgba_unmultiplied(46, 160, 67, 44),
            border: None,
            label: String::new(),
        }
    }

    /// Bounded annular sector intersection `[theta_start, theta_end] x [r_inner, r_outer]`.
    pub fn polar_region(theta_range: [f64; 2], r_range: [f64; 2]) -> Self {
        Self {
            kind: PolarRegionKind::Polar,
            theta_range: Some(theta_range),
            r_range: Some(r_range),
            fill: Color32::from_rgba_unmultiplied(217, 95, 2, 52),
            border: None,
            label: String::new(),
        }
    }

    pub fn fill(mut self, color: Color32) -> Self {
        self.fill = color;
        self
    }

    pub fn border(mut self, stroke: impl Into<Option<Stroke>>) -> Self {
        self.border = stroke.into();
        self
    }

    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = label.into();
        self
    }

    pub fn kind(&self) -> PolarRegionKind {
        self.kind
    }
    pub fn theta_range(&self) -> Option<[f64; 2]> {
        self.theta_range
    }
    pub fn r_range(&self) -> Option<[f64; 2]> {
        self.r_range
    }
    pub fn fill_color(&self) -> Color32 {
        self.fill
    }
    pub fn border_stroke(&self) -> Option<Stroke> {
        self.border
    }
    pub fn label_text(&self) -> &str {
        &self.label
    }

    pub fn is_valid(&self) -> bool {
        let th_ok = match self.theta_range {
            None => true,
            Some([a, b]) => a.is_finite() && b.is_finite(),
        };
        let r_ok = match self.r_range {
            None => true,
            Some([lo, hi]) => lo.is_finite() && hi.is_finite() && hi > lo,
        };
        th_ok && r_ok
    }
}
