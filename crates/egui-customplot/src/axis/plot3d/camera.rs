use egui::{Pos2, Rect, Vec2};

use super::{Bounds3D, DataAspect3D, Point3D};
use crate::{AxisScale, BoundsError};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum Projection3D {
    #[default]
    Orthographic,
    Perspective {
        fov_y_rad: f64,
    },
}

impl Projection3D {
    pub fn perspective_default() -> Self {
        Self::Perspective {
            fov_y_rad: 45.0_f64.to_radians(),
        }
    }
}

/// Camera state matching MATLAB `view(azimuth_deg, elevation_deg)` conventions.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Camera3D {
    pub azimuth_deg: f64,
    pub elevation_deg: f64,
    pub zoom: f64,
    pub pan: [f64; 2],
    pub projection: Projection3D,
}

impl Default for Camera3D {
    fn default() -> Self {
        Self::view_3d()
    }
}

impl Camera3D {
    pub fn new(azimuth_deg: f64, elevation_deg: f64) -> Self {
        Self {
            azimuth_deg,
            elevation_deg: elevation_deg.clamp(-90.0, 90.0),
            zoom: 1.0,
            pan: [0.0, 0.0],
            projection: Projection3D::Orthographic,
        }
    }

    pub fn view_3d() -> Self {
        Self::new(-37.5, 30.0)
    }
    pub fn view_2d_xy() -> Self {
        Self::new(0.0, 90.0)
    }
    pub fn view_xz() -> Self {
        Self::new(0.0, 0.0)
    }
    pub fn view_yz() -> Self {
        Self::new(90.0, 0.0)
    }

    pub fn projection(mut self, projection: Projection3D) -> Self {
        self.projection = projection;
        self
    }

    pub fn basis(self) -> ([f64; 3], [f64; 3], [f64; 3]) {
        let az = self.azimuth_deg.to_radians();
        let el = self.elevation_deg.clamp(-90.0, 90.0).to_radians();
        let (sa, ca) = az.sin_cos();
        let (se, ce) = el.sin_cos();
        let right = [ca, sa, 0.0];
        let up = [-sa * se, ca * se, ce];
        let forward = [-sa * ce, ca * ce, -se];
        (right, up, forward)
    }
}

/// Complete 3D data-to-unit-cube and camera-projection transform.
#[derive(Clone, Copy, Debug)]
pub struct Transform3D {
    bounds: Bounds3D,
    scales: [AxisScale; 3],
    reversed: [bool; 3],
    trans_min: [f64; 3],
    trans_span: [f64; 3],
    box_scale: [f64; 3],
    rect: Rect,
    camera: Camera3D,
    right: [f64; 3],
    up: [f64; 3],
    forward: [f64; 3],
    cam_dist: f64,
    fit_radius: f64,
}

impl Transform3D {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        bounds: Bounds3D,
        scales: [AxisScale; 3],
        reversed: [bool; 3],
        aspect: DataAspect3D,
        rect: Rect,
        camera: Camera3D,
    ) -> Result<Self, BoundsError> {
        if !rect.is_finite() || rect.width() <= 1.0 || rect.height() <= 1.0 {
            return Err(BoundsError);
        }
        let validated = Bounds3D::validated_for_scales(bounds.min(), bounds.max(), scales)?;
        let mut trans_min = [0.0; 3];
        let mut trans_span = [0.0; 3];
        for i in 0..3 {
            let a = scales[i].forward(validated.min()[i]).ok_or(BoundsError)?;
            let b = scales[i].forward(validated.max()[i]).ok_or(BoundsError)?;
            trans_min[i] = a;
            trans_span[i] = b - a;
        }
        let (right, up, forward) = camera.basis();
        let zoom = if camera.zoom.is_finite() {
            camera.zoom.clamp(0.1, 20.0)
        } else {
            1.0
        };
        let cam_dist = match camera.projection {
            Projection3D::Orthographic => 4.0,
            Projection3D::Perspective { fov_y_rad } => {
                let fov = if fov_y_rad.is_finite() {
                    fov_y_rad.clamp(0.15, 2.2)
                } else {
                    45.0_f64.to_radians()
                };
                (1.15 / (fov * 0.5).tan()).max(1.8)
            }
        };
        let fit_radius = f64::from(rect.width().min(rect.height())) * 0.44 * zoom;
        Ok(Self {
            bounds: validated,
            scales,
            reversed,
            trans_min,
            trans_span,
            box_scale: aspect.box_scales(validated),
            rect,
            camera: Camera3D { zoom, ..camera },
            right,
            up,
            forward,
            cam_dist,
            fit_radius,
        })
    }

    pub fn bounds(&self) -> Bounds3D {
        self.bounds
    }
    pub fn scales(&self) -> [AxisScale; 3] {
        self.scales
    }
    pub fn reversed(&self) -> [bool; 3] {
        self.reversed
    }
    pub fn rect(&self) -> Rect {
        self.rect
    }
    pub fn camera(&self) -> Camera3D {
        self.camera
    }
    pub fn box_scale(&self) -> [f64; 3] {
        self.box_scale
    }

    /// Maps data-space `p` to oriented `[0, 1]^3` cube coordinates (can exceed `[0, 1]` before clipping).
    pub fn data_to_unit(&self, p: Point3D) -> Option<[f64; 3]> {
        let mut out = [0.0; 3];
        for i in 0..3 {
            let t = self.scales[i].forward(p[i])?;
            let u = (t - self.trans_min[i]) / self.trans_span[i];
            if !u.is_finite() {
                return None;
            }
            out[i] = if self.reversed[i] { 1.0 - u } else { u };
        }
        Some(out)
    }

    /// Maps oriented `[0, 1]^3` cube coordinates to aspect-scaled world coordinates centered at origin.
    pub fn unit_to_world(&self, u: [f64; 3]) -> [f64; 3] {
        [
            (u[0] - 0.5) * self.box_scale[0],
            (u[1] - 0.5) * self.box_scale[1],
            (u[2] - 0.5) * self.box_scale[2],
        ]
    }

    /// Transforms an aspect-scaled centered world normal/direction into normalized `(right, up, forward)`.
    pub fn data_normal_to_unit_normal(&self, n: [f64; 3]) -> Option<[f64; 3]> {
        let mut out = [0.0; 3];
        for i in 0..3 {
            let s = self.trans_span[i];
            if !n[i].is_finite() || !s.is_finite() {
                return None;
            }
            let signed = if self.reversed[i] { -n[i] } else { n[i] };
            out[i] = signed * s;
        }
        let len = (out[0] * out[0] + out[1] * out[1] + out[2] * out[2]).sqrt();
        if len <= 1e-18 || !len.is_finite() {
            return None;
        }
        Some([out[0] / len, out[1] / len, out[2] / len])
    }

    /// Converts oriented `[0, 1]^3` unit coordinates to `(screen_pos, depth)`. Smaller depth is nearer camera.
    pub fn project_unit(&self, u: [f64; 3]) -> Option<(Pos2, f32)> {
        let w = self.unit_to_world(u);
        let xc = dot(w, self.right) + self.camera.pan[0];
        let yc = dot(w, self.up) + self.camera.pan[1];
        let depth = self.cam_dist + dot(w, self.forward);
        if depth <= 0.12 || !depth.is_finite() {
            return None;
        }
        let persp = match self.camera.projection {
            Projection3D::Orthographic => 1.0,
            Projection3D::Perspective { .. } => self.cam_dist / depth,
        };
        let c = self.rect.center();
        let sx = (f64::from(c.x) + xc * persp * self.fit_radius) as f32;
        let sy = (f64::from(c.y) - yc * persp * self.fit_radius) as f32;
        if sx.is_finite() && sy.is_finite() {
            Some((Pos2::new(sx, sy), depth as f32))
        } else {
            None
        }
    }

    pub fn project_data(&self, p: Point3D) -> Option<(Pos2, f32)> {
        self.data_to_unit(p).and_then(|u| self.project_unit(u))
    }

    pub fn unit_dir_to_screen(&self, u0: [f64; 3], u1: [f64; 3]) -> Option<Vec2> {
        let (p0, _) = self.project_unit(u0)?;
        let (p1, _) = self.project_unit(u1)?;
        let d = p1 - p0;
        d.is_finite().then_some(d)
    }
}

fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn view2d_xy_and_perspective_projection_preserve_orientation_and_depth() {
        let bounds = Bounds3D::new([0.0, 0.0, 0.0], [10.0, 10.0, 10.0]).unwrap();
        let rect = Rect::from_min_max(Pos2::ZERO, Pos2::new(400.0, 400.0));
        let tf_xy = Transform3D::new(
            bounds,
            [AxisScale::Linear; 3],
            [false; 3],
            DataAspect3D::Auto,
            rect,
            Camera3D::view_2d_xy(),
        )
        .unwrap();
        let (o, do_) = tf_xy.project_data([0.0, 0.0, 0.0]).unwrap();
        let (px, _) = tf_xy.project_data([10.0, 0.0, 0.0]).unwrap();
        let (py, _) = tf_xy.project_data([0.0, 10.0, 0.0]).unwrap();
        let (_pz, dz) = tf_xy.project_data([0.0, 0.0, 10.0]).unwrap();
        assert!(px.x > o.x && (px.y - o.y).abs() < 1e-3);
        assert!(py.y < o.y && (py.x - o.x).abs() < 1e-3);
        assert!(dz < do_);

        let tf_persp = Transform3D::new(
            bounds,
            [AxisScale::Linear; 3],
            [false; 3],
            DataAspect3D::Auto,
            rect,
            Camera3D::view_2d_xy().projection(Projection3D::perspective_default()),
        )
        .unwrap();
        let (top_corner, _) = tf_persp.project_data([10.0, 10.0, 10.0]).unwrap();
        let (bot_corner, _) = tf_persp.project_data([10.0, 10.0, 0.0]).unwrap();
        assert!(top_corner.x > bot_corner.x && top_corner.y < bot_corner.y);
    }
}
