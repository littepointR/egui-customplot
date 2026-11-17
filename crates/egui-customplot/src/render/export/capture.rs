use egui::{
    ColorImage, Mesh, Rect, Shape, TextureId, Ui,
    epaint::{ClippedShape, Primitive},
};

use super::{ExportError, ExportOptions, options::additive};

/// Resource bounds enforced while capturing vector/raster shapes for SVG & PNG export.
///
/// 导出快照捕获阶段施加的图元、顶点、索引与字形纹理像素上限，防止异常大图耗尽内存。
#[derive(Clone, Copy, Debug)]
pub struct CaptureLimits {
    pub max_shapes: usize,
    /// Source points and tessellated vertices are checked separately.
    pub max_vertices: usize,
    pub max_indices: usize,
    pub max_font_pixels: usize,
}

impl Default for CaptureLimits {
    fn default() -> Self {
        Self {
            max_shapes: 100_000,
            max_vertices: 4_000_000,
            max_indices: 12_000_000,
            max_font_pixels: 16_777_216,
        }
    }
}

pub(super) struct Item {
    pub clip: Rect,
    pub shape: Shape,
    pub meshes: Vec<Mesh>,
}

/// Host-independent snapshot of one `Painter` layer's newly submitted shapes.
/// Captures DISPLAYED sampling, clipping, font atlas and tessellation. Increasing
/// export resolution cannot recover samples discarded by `MinMax` or relayout text.
/// No `Context`, original samples or `PlotState` is retained. Call once per desired
/// layout pass; save only the final snapshot outside the multi-pass UI closure.
///
/// 与宿主解耦的自包含导出快照：捕获当前布局遍内新提交的绘制图元与字形图集，支持在 UI 闭包外生成确定性矢量 SVG 或 PNG。
pub struct PlotSnapshot {
    pub(super) rect: Rect,
    pub(super) items: Vec<Item>,
    pub(super) font: ColorImage,
    source_pixels_per_point: f32,
}

#[derive(Clone, Copy, Debug, Default)]
struct SnapshotCaptureDepth(usize);

pub(crate) fn is_capturing_snapshot(ctx: &egui::Context) -> bool {
    ctx.data(|d| {
        d.get_temp::<SnapshotCaptureDepth>(egui::Id::NULL)
            .is_some_and(|depth| depth.0 > 0)
    })
}

struct SnapshotCaptureGuard(egui::Context);

impl Drop for SnapshotCaptureGuard {
    fn drop(&mut self) {
        self.0.data_mut(|d| {
            let depth = d
                .get_temp::<SnapshotCaptureDepth>(egui::Id::NULL)
                .unwrap_or_default();
            if depth.0 <= 1 {
                d.remove::<SnapshotCaptureDepth>(egui::Id::NULL);
            } else {
                d.insert_temp(egui::Id::NULL, SnapshotCaptureDepth(depth.0 - 1));
            }
        });
    }
}

impl PlotSnapshot {
    /// Draw exactly once in the EXISTING host Ui, returning the logical export
    /// rectangle (typically `PlotResponse::outer_rect`). Captures only additions
    /// to this Ui's Painter layer, not other layers, tooltips, earlier/later UI or
    /// future shape mutation. The callback runs even if a later limit rejects it.
    /// Layer transforms must be identity; transformed Areas are not supported.
    pub fn capture(
        ui: &mut Ui,
        limits: CaptureLimits,
        draw: impl FnOnce(&mut Ui) -> Rect,
    ) -> Result<Self, ExportError> {
        let layer = ui.layer_id();
        if ui
            .ctx()
            .layer_transform_to_global(layer)
            .is_some_and(|t| t != egui::emath::TSTransform::IDENTITY)
        {
            return Err(ExportError::InvalidGeometry);
        }
        let start = ui
            .ctx()
            .graphics(|g| g.get(layer).map_or(0, |l| l.next_idx().0));
        ui.ctx().data_mut(|d| {
            let depth = d
                .get_temp::<SnapshotCaptureDepth>(egui::Id::NULL)
                .unwrap_or_default();
            d.insert_temp(egui::Id::NULL, SnapshotCaptureDepth(depth.0 + 1));
        });
        let rect = {
            let _guard = SnapshotCaptureGuard(ui.ctx().clone());
            draw(ui)
        };
        if !rect.is_finite() || !rect.is_positive() {
            return Err(ExportError::InvalidDimensions);
        }
        let shapes = ui.ctx().graphics(|g| {
            let mut shapes = Vec::new();
            let mut source = SourceBudget::default();
            if let Some(list) = g.get(layer) {
                for clipped in list.all_entries().skip(start) {
                    if clipped.transform_after_tessellation != egui::emath::TSTransform::IDENTITY {
                        return Err(ExportError::InvalidGeometry);
                    }
                    collect(
                        &clipped.shape,
                        clipped.clip_rect.intersect(rect),
                        limits,
                        &mut source,
                        &mut shapes,
                        0,
                    )?;
                }
            }
            Ok::<_, ExportError>(shapes)
        })?;
        let ppp = ui.ctx().pixels_per_point();
        let mut items = Vec::new();
        let (mut vertices, mut indices) = (0usize, 0usize);
        for clipped in shapes {
            let primitives = ui.ctx().tessellate(vec![clipped.clone()], ppp);
            let mut meshes = Vec::new();
            for primitive in primitives {
                let Primitive::Mesh(mesh) = primitive.primitive else {
                    return Err(ExportError::UnsupportedCallback);
                };
                vertices = vertices.saturating_add(mesh.vertices.len());
                indices = indices.saturating_add(mesh.indices.len());
                if vertices > limits.max_vertices || indices > limits.max_indices {
                    return Err(ExportError::LimitExceeded("tessellated geometry"));
                }
                validate_mesh(&mesh)?;
                meshes.push(mesh);
            }
            items.push(Item {
                clip: clipped.clip_rect,
                shape: clipped.shape,
                meshes,
            });
        }
        let font = ui.ctx().fonts(|f| {
            let size = f.font_image_size();
            if size[0]
                .checked_mul(size[1])
                .is_none_or(|n| n > limits.max_font_pixels)
            {
                Err(ExportError::LimitExceeded("font atlas pixels"))
            } else {
                Ok(f.image())
            }
        })?;
        Ok(Self {
            rect,
            items,
            font,
            source_pixels_per_point: ppp,
        })
    }

    pub fn logical_rect(&self) -> Rect {
        self.rect
    }
    pub fn source_pixels_per_point(&self) -> f32 {
        self.source_pixels_per_point
    }

    /// CPU gamma-space premultiplied compositing; not a GPU screenshot.
    pub fn to_png(&self, options: ExportOptions) -> Result<Vec<u8>, ExportError> {
        let dimensions = options.dimensions(self.rect)?;
        let pixels = super::raster::render(self, options, dimensions)?;
        super::png::encode(&pixels, dimensions, options)
    }

    /// Hybrid SVG: vector paths / rectangles, solid meshes, and embedded PNG for
    /// host-font text or varying-color/textured meshes. No external fonts/assets.
    pub fn to_svg(&self, options: ExportOptions) -> Result<Vec<u8>, ExportError> {
        super::svg::encode(self, options)
    }
}

#[derive(Default)]
struct SourceBudget {
    vertices: usize,
    indices: usize,
}

fn collect(
    shape: &Shape,
    clip: Rect,
    limits: CaptureLimits,
    source: &mut SourceBudget,
    output: &mut Vec<ClippedShape>,
    depth: usize,
) -> Result<(), ExportError> {
    if !clip.is_positive() {
        return Ok(());
    }
    if depth > 64 {
        return Err(ExportError::LimitExceeded("nested shapes"));
    }
    if let Shape::Vec(shapes) = shape {
        for shape in shapes {
            collect(shape, clip, limits, source, output, depth + 1)?;
        }
        return Ok(());
    }
    let (vertices, indices) = match shape {
        Shape::Noop => return Ok(()),
        Shape::Callback(_) => return Err(ExportError::UnsupportedCallback),
        Shape::Mesh(mesh) => {
            validate_mesh(mesh)?;
            (mesh.vertices.len(), mesh.indices.len())
        }
        Shape::Path(path) => {
            if path.points.iter().any(|p| !p.is_finite()) {
                return Err(ExportError::InvalidGeometry);
            }
            (path.points.len(), 0)
        }
        Shape::Text(text) => text
            .galley
            .rows
            .iter()
            .fold((0usize, 0usize), |(v, i), row| {
                (
                    v.saturating_add(row.visuals.mesh.vertices.len()),
                    i.saturating_add(row.visuals.mesh.indices.len()),
                )
            }),
        _ => (1, 0),
    };
    source.vertices = source.vertices.saturating_add(vertices);
    source.indices = source.indices.saturating_add(indices);
    if output.len() >= limits.max_shapes
        || source.vertices > limits.max_vertices
        || source.indices > limits.max_indices
    {
        return Err(ExportError::LimitExceeded("source shapes/points"));
    }
    output.push(ClippedShape::new(clip, shape.clone()));
    Ok(())
}

fn validate_mesh(mesh: &Mesh) -> Result<(), ExportError> {
    if mesh.texture_id != TextureId::default() {
        return Err(ExportError::UnsupportedTexture(mesh.texture_id));
    }
    if !mesh.is_valid()
        || !mesh.indices.len().is_multiple_of(3)
        || mesh
            .vertices
            .iter()
            .any(|v| !v.pos.is_finite() || !v.uv.is_finite())
    {
        return Err(ExportError::InvalidGeometry);
    }
    if mesh.vertices.iter().any(|v| additive(v.color)) {
        return Err(ExportError::UnsupportedAdditiveColor);
    }
    Ok(())
}
