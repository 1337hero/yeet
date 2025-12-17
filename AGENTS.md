# Yeet App Launcher

## Project Overview

- Yeet is a minimal, fast, configurable app launcher for Linux (Wayland), inspired by wofi/walker.
- UI: Rust + GTK4-rs; Wayland layer surface via `gtk4-layer-shell` (overlay, keyboard grab).
- Config: TOML + CSS in `~/.config/yeet/` with embedded defaults in `defaults/`.
- Search: `fuzzy-matcher` (skim backend).
- Designed for Hyprland/Sway; supports compositor blur where available.

## Project Structure & Module Organization

- `src/`: application code
  - `main.rs`: GTK application entrypoint
  - `ui.rs`: window layout, search, keyboard handling
  - `desktop.rs`: `.desktop` discovery + launching
  - `config.rs`: config loading/merge from defaults + user overrides
- `defaults/`: embedded runtime defaults (`config.toml`, `style.css`); ships an embedded default theme (currently Catppuccin Macchiato); user overrides live in `~/.config/yeet/`.
- `.github/workflows/`: CI (`ci.yml`) and tag-based releases (`release.yml`).
- Packaging/metadata: `PKGBUILD` and `Cargo.toml` (`package.metadata.deb`).

## Build, Test, and Development Commands

This is a single-binary Rust crate. Toolchain is pinned in `rust-toolchain.toml` (Rust `1.85`, `rustfmt`, `clippy`). GTK4 development libraries are required (e.g. `libgtk-4-dev` on Debian/Ubuntu).

- `just` / `just --list`: list common tasks.
- `just run` (`cargo run`): build + run (debug).
- `just run-release` (`cargo run --release`): run optimized build.
- `cargo build -r`: release build (equivalent to `cargo build --release`).
- `just check`: format-check, clippy (deny warnings), and tests (mirrors CI).
- `just fmt`, `just lint`, `just clean`, `just install`, `just init-config`: common maintenance/setup tasks.

## Coding Style & Naming Conventions

- Run `cargo fmt` before committing; CI enforces `cargo fmt --check`.
- Treat clippy warnings as errors: `cargo clippy --all-targets -- -D warnings`.
- Follow Rust conventions: `snake_case` for modules/functions, `CamelCase` for types, `SCREAMING_SNAKE_CASE` for constants.
- Keep cross-cutting changes minimal: UI in `src/ui.rs`, app discovery in `src/desktop.rs`, config semantics in `src/config.rs`.

## Rust / Design Philosophy

- Ship apps, not frameworks: keep a single binary crate; avoid unnecessary crate splits.
- Concrete over generic: start with structs/functions; add traits/generics only when they clearly simplify.
- Own at boundaries, borrow internally: prefer `String`/`Vec` in public APIs; keep lifetimes internal.
- Sync by default: introduce async only when latency/concurrency actually demands it.
- Errors should be boring: propagate with `?` and add context when useful.
- One binary, simple config: avoid daemons/services and keep config in `~/.config/yeet/`.
- Measure before optimizing: ship, profile, then fix hot paths; avoid pre-paying with complexity.
- Clippy is a guide: prioritize readability/intent; satisfy CI by fixing lints or narrowly `#[allow]` when a lint hurts clarity.
- Duplication beats wrong abstraction: refactor once the shape stabilizes.
- Naming is design: prefer descriptive names over comments; extract helpers instead of annotating.
- Stay on stable: avoid nightly features and keep maintenance low.

## Testing Guidelines

- Run tests with `cargo test` (or `just check`).
- Add unit tests in-module (`#[cfg(test)] mod tests { ... }`) and integration tests under `tests/` when behavior crosses modules.

## Commit & Pull Request Guidelines

- Git history is currently minimal (e.g. `init commit`); use short, imperative subjects and include a scope when helpful (example: `ui: close on Escape`).
- PRs should describe what/why, note any config/default changes (`defaults/`), and include screenshots for UI/CSS changes.
- Keep build artifacts out of commits (`target/`).

## Distribution

- Primary: AUR (`yeet-git`).
- Secondary: GitHub releases with prebuilt binaries.
