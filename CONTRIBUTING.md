# Contributing to egui-customplot

Thank you for your interest in contributing to `egui-customplot`!

Please review `AGENTS.md` and `docs/ARCHITECTURE.md` before making changes so new features stay aligned with the library's design guarantees and automated checks.

## Core Design & Architecture Rules

1. **Seven Source Domains**: All library implementation files under `crates/egui-customplot/src/` live in one of seven domain directories (`axis`, `data`, `interaction`, `layout`, `model`, `render`, `widget`) plus `diagnostics.rs` and `lib.rs`.
   - `lib.rs` handles crate docs, private module declarations, and public re-exports.
   - `widget` only orchestrates per-pass stages; coordinate math, data scanning, picking, and tessellation belong in their respective domains.
   - Avoid catch-all module names such as `utils.rs`, `common.rs`, or `helpers.rs`.
2. **File Size Guardrail (`tests/architecture.rs`)**: Keep library source files focused by responsibility; `tests/architecture.rs` rejects any `.rs` file in `crates/egui-customplot/src/` exceeding **400 lines**. Split cleanly by responsibility rather than compressing formatting.
3. **No `unsafe` Code**: `[workspace.lints.rust] unsafe_code = "forbid"` applies across all crates.
4. **Host Ownership & Multi-Pass Idempotency**:
   - Raw samples and host caches belong to the caller (`Arc` payloads, `PlotCache`, `PlotStream`), never inside `PlotState` or `egui::Memory`.
   - Do not spawn windows, event loops, hidden background threads, network calls, or private `wgpu::Device` instances in the core library.
   - All interaction, `AxisLink` syncing, streaming appends, and file side-effects must deduplicate cleanly across discarded `egui::Context::request_discard` layout passes.
5. **Documentation & Public API**:
   - Keep `crate`-root re-exports backwards compatible; keep internal submodules private or `pub(crate)`.
   - Document public items with concise English + Simplified Chinese doc comments (compatible with the interactive `EN | 简体中文` switcher in `crates/egui-customplot/docs/rustdoc-i18n-header.html`), and verify zero `rustdoc` warnings.

## Local Verification Checklist

Before opening a pull request, run the full verification suite:

```sh
cargo fmt --all -- --check
cargo test --workspace --locked
cargo test --workspace --all-features --locked
cargo check -p egui-customplot --no-default-features --locked
cargo doc -p egui-customplot --no-deps --no-default-features --locked
cargo doc --workspace --no-deps --all-features --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo build -p customplot-demo --release --locked
```

## Benchmark & Evidence Discipline

- Never overwrite existing files in `docs/benchmarks/`. Record new benchmark outputs under a new dated filename with environment details.
- Clearly distinguish headless CPU + tessellation timings from native swapchain/GPU frame timings.
- Keep `publish = false` in `examples/demo/Cargo.toml` and do not bump versions, trigger crates.io releases, or alter the frozen `docs/MATLAB_PLOT_BASELINE.json` / `docs/MATLAB_PLOT_CHECKLIST.md` 133-item denominator unless explicitly requested.

## Licensing

`egui-customplot` is dual-licensed under `MIT OR Apache-2.0`. By submitting a contribution, you agree that your contribution is licensed under both the MIT License and the Apache License, Version 2.0 (Apache-2.0 Section 5), without any additional terms or conditions.
