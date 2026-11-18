## Summary

Describe what changed and why.

## Checklist

- [ ] Code respects the 7-domain layout in `docs/ARCHITECTURE.md` and `<= 400` lines per library source file (`tests/architecture.rs`)
- [ ] Workspace compiles with `#![forbid(unsafe_code)]` and passes `cargo fmt --all -- --check`
- [ ] Tests pass for both `--locked` and `--all-features --locked` (including `rustdoc` / doctests)
- [ ] `cargo check -p egui-customplot --no-default-features --locked` and `cargo doc` produce zero warnings
- [ ] `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings` passes
- [ ] Public API docs / guides updated (EN & 简体中文 where applicable), and any new benchmarks recorded separately without overwriting `docs/benchmarks/*`
