# CPU 基线记录

- [2026-11-15 Windows 首份基线](2026-11-15-windows-cpu.csv)
- 条件、解释与限制见 [阶段 1 记录](../STAGE_1.md#首份-cpu-基线)。

记录来自优化构建，每个场景预热 30 帧、采样至少 30 秒。耗时列为全部采样的分位数；访问段、输出点、几何容量和 mesh 顶点数是**最后一帧快照**，不是平均数或累计值。它不是实际窗口 FPS 或 GPU 基准，不能作为已验证的占用预算。

重新采集（PowerShell）：

```powershell
cargo bench -p egui-customplot --bench frame_baseline --locked |
    Tee-Object -FilePath docs/benchmarks/new-run.csv
```

保留原始记录，后续实验另取文件名；记录 CPU、系统、工具链和任何不同设置，不把短时试跑覆盖为正式基线。
