use egui::{ColorImage, Pos2, Rect, epaint::Vertex};

use super::{ExportError, ExportOptions, PlotSnapshot, capture::Item};

pub(super) fn render(
    scene: &PlotSnapshot,
    options: ExportOptions,
    size: [usize; 2],
) -> Result<Vec<u8>, ExportError> {
    render_items(scene, &scene.items, scene.rect, size, options, &mut 0)
}

pub(super) fn render_items(
    scene: &PlotSnapshot,
    items: &[Item],
    rect: Rect,
    size: [usize; 2],
    options: ExportOptions,
    steps: &mut u64,
) -> Result<Vec<u8>, ExportError> {
    let count = size[0]
        .checked_mul(size[1])
        .ok_or(ExportError::InvalidDimensions)?;
    if count > options.max_pixels {
        return Err(ExportError::LimitExceeded("output pixels"));
    }
    let mut pixels = Vec::new();
    pixels
        .try_reserve_exact(count)
        .map_err(|_| ExportError::AllocationFailed)?;
    let background = options.background.to_array().map(|v| v as f32 / 255.0);
    pixels.resize(count, background);
    let mut canvas = Canvas {
        pixels,
        size,
        rect,
        options,
        steps,
        font: &scene.font,
    };
    for item in items {
        let clip = item.clip.intersect(rect);
        if !clip.is_positive() {
            continue;
        }
        for mesh in &item.meshes {
            for indices in mesh.triangles() {
                canvas.triangle(indices.map(|i| mesh.vertices[i as usize]), clip)?;
            }
        }
    }
    let mut rgba = Vec::new();
    rgba.try_reserve_exact(count.checked_mul(4).ok_or(ExportError::AllocationFailed)?)
        .map_err(|_| ExportError::AllocationFailed)?;
    for pixel in canvas.pixels {
        let alpha = pixel[3].clamp(0.0, 1.0);
        for color in &pixel[..3] {
            rgba.push(if alpha > 0.0 {
                (color / alpha * 255.0).round().clamp(0.0, 255.0) as u8
            } else {
                0
            });
        }
        rgba.push((alpha * 255.0).round() as u8);
    }
    Ok(rgba)
}

struct Canvas<'a> {
    pixels: Vec<[f32; 4]>,
    size: [usize; 2],
    rect: Rect,
    options: ExportOptions,
    steps: &'a mut u64,
    font: &'a ColorImage,
}

impl Canvas<'_> {
    fn triangle(&mut self, mut vertices: [Vertex; 3], clip: Rect) -> Result<(), ExportError> {
        let ppp = self.options.pixels_per_point as f64;
        let mut p = vertices.map(|v| {
            [
                (v.pos.x as f64 - self.rect.min.x as f64) * ppp,
                (v.pos.y as f64 - self.rect.min.y as f64) * ppp,
            ]
        });
        let mut area = edge(p[0], p[1], p[2]);
        if area == 0.0 {
            return Ok(());
        }
        if area < 0.0 {
            p.swap(1, 2);
            vertices.swap(1, 2);
            area = -area;
        }
        let clip_min = [
            (clip.min.x as f64 - self.rect.min.x as f64) * ppp,
            (clip.min.y as f64 - self.rect.min.y as f64) * ppp,
        ];
        let clip_max = [
            (clip.max.x as f64 - self.rect.min.x as f64) * ppp,
            (clip.max.y as f64 - self.rect.min.y as f64) * ppp,
        ];
        let min: [usize; 2] = std::array::from_fn(|axis| {
            p.iter()
                .map(|p| p[axis])
                .fold(f64::INFINITY, f64::min)
                .max(clip_min[axis])
                .floor()
                .clamp(0.0, self.size[axis] as f64) as usize
        });
        let max: [usize; 2] = std::array::from_fn(|axis| {
            p.iter()
                .map(|p| p[axis])
                .fold(f64::NEG_INFINITY, f64::max)
                .min(clip_max[axis])
                .ceil()
                .clamp(0.0, self.size[axis] as f64) as usize
        });
        let work = (max[0].saturating_sub(min[0]) as u64)
            .saturating_mul(max[1].saturating_sub(min[1]) as u64);
        *self.steps = self.steps.saturating_add(work);
        if *self.steps > self.options.max_raster_steps {
            return Err(ExportError::LimitExceeded("raster work"));
        }
        let colors = vertices.map(|v| v.color.to_array().map(|v| v as f32 / 255.0));
        for y in min[1]..max[1] {
            for x in min[0]..max[0] {
                let center = [x as f64 + 0.5, y as f64 + 0.5];
                if (0..2).any(|i| center[i] < clip_min[i] || center[i] >= clip_max[i]) {
                    continue;
                }
                let e = [
                    edge(p[1], p[2], center),
                    edge(p[2], p[0], center),
                    edge(p[0], p[1], center),
                ];
                // Half-open edges prevent alpha doubling on shared diagonals.
                if (0..3).any(|i| {
                    e[i] < 0.0 || (e[i] == 0.0 && !top_left(p[(i + 1) % 3], p[(i + 2) % 3]))
                }) {
                    continue;
                }
                let weights = e.map(|e| (e / area) as f32);
                let uv = egui::pos2(
                    (0..3).map(|i| vertices[i].uv.x * weights[i]).sum(),
                    (0..3).map(|i| vertices[i].uv.y * weights[i]).sum(),
                );
                let texel = sample(self.font, uv);
                let source: [f32; 4] = std::array::from_fn(|channel| {
                    texel[channel] * (0..3).map(|i| colors[i][channel] * weights[i]).sum::<f32>()
                });
                let destination = &mut self.pixels[y * self.size[0] + x];
                for channel in 0..4 {
                    destination[channel] =
                        source[channel] + destination[channel] * (1.0 - source[3]);
                }
            }
        }
        Ok(())
    }
}

fn edge(a: [f64; 2], b: [f64; 2], p: [f64; 2]) -> f64 {
    (b[0] - a[0]) * (p[1] - a[1]) - (b[1] - a[1]) * (p[0] - a[0])
}

fn top_left(a: [f64; 2], b: [f64; 2]) -> bool {
    b[1] < a[1] || (b[1] == a[1] && b[0] > a[0])
}

fn sample(image: &ColorImage, uv: Pos2) -> [f32; 4] {
    let x = (uv.x * image.size[0] as f32 - 0.5).clamp(0.0, (image.size[0] - 1) as f32);
    let y = (uv.y * image.size[1] as f32 - 0.5).clamp(0.0, (image.size[1] - 1) as f32);
    let (x0, y0) = (x.floor() as usize, y.floor() as usize);
    let (x1, y1) = (
        (x0 + 1).min(image.size[0] - 1),
        (y0 + 1).min(image.size[1] - 1),
    );
    let (tx, ty) = (x.fract(), y.fract());
    let colors = [(x0, y0), (x1, y0), (x0, y1), (x1, y1)]
        .map(|(x, y)| image.pixels[y * image.size[0] + x].to_array());
    std::array::from_fn(|i| {
        ((colors[0][i] as f32 * (1.0 - tx) + colors[1][i] as f32 * tx) * (1.0 - ty)
            + (colors[2][i] as f32 * (1.0 - tx) + colors[3][i] as f32 * tx) * ty)
            / 255.0
    })
}
