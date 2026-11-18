//! Camera-space navigation policy. Input deltas belong to one logical egui frame.
use crate::Camera3D;

#[derive(Clone, Copy, Debug, Default)]
pub struct CameraGesture3D {
    pub orbit_points: Option<[f32; 2]>,
    pub pan_points: Option<[f32; 2]>,
    pub scroll_points: Option<f32>,
}

/// Return the camera for this frame. Layout retries replay the same starting camera.
/// Implementations should be pure and should return finite camera parameters.
pub trait CameraNavigation3D: Send + Sync {
    fn camera(
        &self,
        previous: Camera3D,
        gesture: CameraGesture3D,
        viewport: egui::Rect,
    ) -> Camera3D;
}

pub(super) struct OrbitNavigation;
impl CameraNavigation3D for OrbitNavigation {
    fn camera(&self, mut camera: Camera3D, gesture: CameraGesture3D, rect: egui::Rect) -> Camera3D {
        if let Some([dx, dy]) = gesture.orbit_points {
            camera.azimuth_deg = (camera.azimuth_deg - f64::from(dx) * 0.45).rem_euclid(360.);
            camera.elevation_deg = (camera.elevation_deg + f64::from(dy) * 0.45).clamp(-89.5, 89.5);
        }
        if let Some([dx, dy]) = gesture.pan_points {
            let scale = f64::from(rect.width().min(rect.height())).max(40.) * 0.44 * camera.zoom;
            camera.pan[0] = (camera.pan[0] + f64::from(dx) / scale).clamp(-2.5, 2.5);
            camera.pan[1] = (camera.pan[1] - f64::from(dy) / scale).clamp(-2.5, 2.5);
        }
        if let Some(delta) = gesture.scroll_points {
            camera.zoom =
                (camera.zoom * (f64::from(delta) / 220.).exp().clamp(0.5, 2.)).clamp(0.15, 20.);
        }
        camera
    }
}
