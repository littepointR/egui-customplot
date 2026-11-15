# egui-customplot

以 MATLAB 的科学与工程绘图体验为参考，基于 Rust / egui 构建高性能、可定制的绘图库。

**当前状态：规划阶段，尚无可运行实现。**

首期聚焦二维交互绘图；借鉴 Figure / Axes / Series、属性配置与数据探索体验，不追求 MATLAB 语法兼容。

- [项目规划与分阶段验收](docs/PLAN.md)
- [MATLAB 绘图类型完整交付对照](docs/MATLAB_PLOT_CHECKLIST.md)：R2026b 官方总览 82 项，另含本体补充与传统接口；工具箱扩展单列。
- [egui 集成、性能与资源占用约束](docs/EGUI_INTEGRATION_AND_PERFORMANCE.md)：原生嵌入、多遍布局、按需重绘、CPU / GPU 预算与验收要求，当前均未验证。
- 推荐推进顺序：最小闭环 → 绘图能力 → 定制能力 → 数据性能 → 按需 GPU 加速 → 导出与发布。

平台、功能范围和性能指标均为待确认的建议，不代表已经实现或测得的结果。
