# 阶段 7：显式显示快照与 PNG / 混合 SVG

日期：2026-11-15。**导出基础切片已实现，完整二维交付及平台验收未完成。**

## 接入与采样契约

库默认仍仅依赖 egui；启用 `export` feature 才引入 PNG 编码器。`PlotSnapshot::capture` 在已有宿主 Ui 中执行一次绘图闭包，返回当前 Painter layer 在闭包期间新增的形状、裁剪和字体 atlas 快照；不创建新窗口 / Context / GPU device，不持有原始样本或 PlotState。`PlotResponse::outer_rect` 包含标题、刻度和轴标签，可用作导出范围。

```rust,ignore
let snapshot = PlotSnapshot::capture(ui, CaptureLimits::default(), |ui| {
    Plot::new("signal").show(ui, state, &[Line::new(data)]).outer_rect
})?;
// 在最终布局遍完成后，由宿主保存最终 snapshot，而不是每遍写文件：
let bytes = snapshot.to_png(ExportOptions::default())?;
```

这是**显示快照**，不是从样本重建的无损数据文件。Exact 绘图捕获 Exact 几何，MinMax 绘图只捕获其显式近似几何；增大输出尺寸不能恢复删减的点。需要 Exact 时，宿主必须先选择 `LineSampling::Exact` 绘图再捕获，并承担对应成本。不能以此替代重新布局到印刷尺寸的高质量矢量导出。

只捕获该 Ui 的原 layer；工具提示、其他 layer、闭包前后 UI 和之后的 shape 修改均不包含。非恒等 layer / ClippedShape 变换明确拒绝；不支持变换中的 Area。字体快照不会消费宿主的 texture upload delta。每个形状使用宿主 `Context::tessellate`，会临时更新 egui paint stats，正常帧 tessellation 随后覆盖它；不得将导出开销算入普通绘图基准。

## 像素、透明度与字体策略

- PNG：CPU 三角形栅格化、双线性字体 atlas 采样、预乘 gamma-space source-over；输出非预乘 RGBA8 / sRGB。半开三角边避免共享对角线重复 alpha。不是 GPU 截屏，不保证与 Glow / wgpu 逐像素一致。
- SVG：常见矩形、圆和中线描边的 path 保留矢量；纯色无纹理 mesh 为矢量三角形。字形、旋转文字、下划线、渐变顶点 / AA mesh 及其他形状经已捕获的 tessellation 栅格化，作为带 alpha 的 PNG patch 内嵌。保留 clipPath 和提交顺序，不引用外部图片 / 字体。
- SVG 是**混合矢量**，不是全部转路径的论文级输出；文字不可搜索或编辑，放大仍受原字体 atlas 分辨率限制。宿主缺字会保留其实际替代字形，导出不会自行寻找或下载字体。需要中文应由宿主注入适当字体后再布局。
- `pixels_per_point` 控制 `ceil(logical_size × scale)` 像素尺寸，不重新查询数据 / 重排文字。SVG viewBox 使用相同像素网格，非整数尺寸的补齐不拉伸内容。
- `dpi` 仅写 PNG pHYs 物理分辨率元数据，不改变像素数或 SVG 尺寸。默认 96 DPI、1 pixel / point；两者不是同一参数。
- 背景在所有捕获内容之后垫底，可透明；它不会擦除已经绘制的 Plot background。需要全透明图需显式设置 PlotStyle 的 background。
- 自定义 GPU callback、非字体纹理和加色混合明确返回错误，不能静默遗漏或假装支持。数据和字形最终使用相同捕获 clip；SVG 原生 stroke / join / 像素对齐仍可能与 epaint 略有差异。

## 局部资源护栏

`CaptureLimits` 限制形状数、源点 / tessellated 顶点、索引和字体像素。源 mesh 在复制前校验；tessellation 产生的几何在生成后检查，因此不是临时分配或进程峰值硬上限。宿主绘图已发生，即使捕获返回错误也不回滚或重放闭包。

`ExportOptions` 默认最多 4,194,304 像素、64 MiB 编码输出、200,000,000 次三角形包围盒像素访问；后者包含过度绘制与未覆盖的包围盒点。PNG 画布使用 f32 RGBA，另有 RGBA8 / 编码缓冲；SVG 局部 patch 也受像素与累计栅格工作限制。输出写入器在超出编码字节限制前停止；这些不是全局内存 / GPU / 固定时间保证。

库只返回 bytes，不隐式写盘。demo 保存时使用 create_new 拒绝覆盖已有文件；每帧最后一次候选快照替换前一遍，到下一帧再写盘一次，不在 discarded pass 产生文件副作用。路径可在面板编辑，默认 `target/exports/plot.png` 或 `.svg`。

## 运行、测试与未完成项

```sh
cargo run -p customplot-demo --release --locked
# 面板 Export displayed gallery，可导出三个 gallery 的当前显示。
cargo run -p egui-customplot --example export_snapshot --features export --release --locked -- target/exports/example
cargo test --workspace --all-features --locked
cargo check -p egui-customplot --no-default-features --locked
```

独立 example 是一个 headless egui 宿主，不是库私建第二个 Context。实跑生成 `target/exports/2026-11-15-stage7.png`（20,071 B）与 `.svg`（21,972 B），示例选择 Exact；历史文件不覆盖。

专项 `export_pixels` 4 项、`export_fonts` 1 项、`export_contract` 4 项、`export_sampling` 2 项，含真实 PNG 解码、RGBA 像素 / pHYs / sRGB、实际文件、XML 解析、resvg 无外部字体渲染、旋转字形 / 颜色覆盖 / 下划线、透明叠加 / 裁剪 / 共享边、LOD 不恢复、原始样本释放和限制拒绝。resvg / roxmltree 仅为测试依赖，不进入库默认依赖。demo 另有三 gallery 导出与延迟文件 / 防覆盖测试。

最终统一批次 161 项通过（库单元 32、集成 118、demo 9、rustdoc 2），fmt、无默认 feature 库检查、严格 all-targets / all-features Clippy、Release demo 构建通过。最新 binary SHA256 `A6D7313C05CB79B5EF941E052A0C9753CCE50CC3C7F92BEB9A9FFD27C6BF110C` 的三个 Windows gallery 启动、响应与正常关闭均通过，见 [独立 native smoke](benchmarks/2026-11-15-stage7-windows-gallery-smoke.txt)。这仅是窗口生命周期，不是人工视觉、GPU 时间、FPS 或整应用长期资源验收；阶段 6 的旧 binary 记录保留不覆盖。

尚未完成：独立印刷尺寸与高 DPI 字体重排、全部文字转路径、所有 layer / 外部纹理、更多导出基准 / 长期资源、跨平台实机视觉与完整发布准备。许可证仍未决定，仓库保持私有、publish=false；不发布 crates.io。阶段 8～11 和 MATLAB 133 项完整核销仍保留。
