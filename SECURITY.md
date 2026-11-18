# Security Policy

## Supported Versions

`egui-customplot` is currently in the `0.1.x` release series on the `main` branch. Security fixes are applied directly to `main`.

## Safety & Isolation Design

- **Memory safety**: `#![forbid(unsafe_code)]` is enforced workspace-wide (`[workspace.lints.rust] unsafe_code = "forbid"`).
- **Network & filesystem isolation**: The core `egui-customplot` crate performs **zero** network requests, spawns no hidden background threads, and performs no implicit filesystem writes (`export` returns in-memory byte buffers to the caller).

## Reporting a Vulnerability

Please **do not** open a public issue for undisclosed security vulnerabilities.

Instead, please report security issues privately using **GitHub Security Advisories** ("Report a vulnerability" under the repository's **Security** tab), including:

1. A clear description of the issue and affected component or Cargo feature (`default`, `export`, `wgpu`).
2. Steps or minimal reproduction code demonstrating the behavior.
3. Expected vs. actual behavior and any relevant platform/driver details.
