//! Projective camera lenses with an analytic inverse for picking and volume rays.
use crate::BoundsError;

/// A projective lens in camera coordinates. The first two rows map `[x, y, depth, 1]`
/// to image numerators. The denominator is linear in depth. Its coefficients must be nonnegative and cannot both be zero. The XY block must
/// be invertible. Image coordinates are in camera units before viewport fitting.
#[derive(Clone, Copy, Debug)]
pub struct CameraLens3D {
    pub(super) rows: [[f64; 4]; 2],
    pub(super) divisor: [f64; 2],
    inv_det: f64,
}

impl CameraLens3D {
    pub fn new(rows: [[f64; 4]; 2], divisor: [f64; 2]) -> Result<Self, BoundsError> {
        let det = rows[0][0] * rows[1][1] - rows[0][1] * rows[1][0];
        if !rows
            .iter()
            .flatten()
            .chain(divisor.iter())
            .all(|v| v.is_finite())
            || divisor.iter().any(|v| *v < 0.)
            || divisor == [0., 0.]
            || !det.is_finite()
            || !det.recip().is_finite()
        {
            return Err(BoundsError);
        }
        Ok(Self {
            rows,
            divisor,
            inv_det: det.recip(),
        })
    }
    pub(super) fn project(self, x: f64, y: f64, z: f64) -> Option<[f64; 2]> {
        let w = self.divisor[0] * z + self.divisor[1];
        (w.is_finite() && w > 0.).then(|| {
            self.rows
                .map(|r| (r[0] * x + r[1] * y + r[2] * z + r[3]) / w)
        })
    }
    pub(super) fn ray(self, sx: f64, sy: f64) -> ([f64; 2], [f64; 2]) {
        let [a, b] = self.rows;
        let solve = |x, y| {
            [
                (b[1] * x - a[1] * y) * self.inv_det,
                (a[0] * y - b[0] * x) * self.inv_det,
            ]
        };
        (
            solve(sx * self.divisor[1] - a[3], sy * self.divisor[1] - b[3]),
            solve(sx * self.divisor[0] - a[2], sy * self.divisor[0] - b[2]),
        )
    }
    pub(crate) fn coefficients(self) -> impl Iterator<Item = f64> {
        self.rows.into_iter().flatten().chain(self.divisor)
    }
}
