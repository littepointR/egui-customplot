# 开发约定

- 先读 `docs/PROGRESS.md`、`docs/ARCHITECTURE.md`，按阶段推进；没有测试 / 证据的能力不得标为完成。
- 库源码按 `data / model / axis / layout / interaction / render / widget` 分域；`lib.rs` 只负责入口、文档和公开导出。不要重新创建平铺的 `model.rs`、`render.rs` 或大型 `widget.rs`。
- `widget` 仅组装阶段，不放坐标算法、原始数据扫描、拾取或几何细节。新增能力放入所属域；不要建无明确职责的 `utils.rs`、`common.rs`、`helpers.rs`。
- 单文件以职责为边界；接近 300 行时评估拆分，源码 400 行检查作为回归护栏，不是鼓励塞满 400 行。不要通过压缩代码或绕过测试规避检查。
- 保持 crate 根公开 API 兼容；内部模块默认私有，尽量使用受限可见性。当前不按每个目录另拆 crate。
- 样本由宿主持有，`PlotState` 不持有数据；不得另建窗口、Context、事件循环或 GPU device。处理 egui 多遍布局时，输入 / 追加 / 事件副作用只应用一次。
- 演示程序的启动、应用组装、控件、数据和诊断分文件；测试共用代码放 `tests/support/`，按功能添加集成测试。
- 检查：`cargo fmt --all -- --check`、`cargo test --workspace --locked`、`cargo clippy --workspace --all-targets --locked -- -D warnings`、`cargo doc --workspace --no-deps --all-features --locked`、Release demo 构建。离线环境可附加 `--offline`。
- 历史性能记录不能覆盖；新结果独立记录条件。CPU 构建 / tessellation 不是 GPU 或 FPS。未完成项继续保留在清单。
- 许可证与 egui 一致采用 `MIT OR Apache-2.0`（`LICENSE-MIT` / `LICENSE-APACHE`）；包保持 `publish = false`，未经明确指示不得发布 crates.io。
