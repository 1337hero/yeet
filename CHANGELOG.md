# Changelog

All notable changes to this project will be documented in this file.

## [Unreleased]

### Added
- Desktop entry names, descriptions, and keywords now honor the system locale instead of always using English

### Fixed
- Recently launched apps now surface in the initial list even when they sort outside the first `initial_results` alphabetically
- Launching yeet while a window is already open no longer stacks a second window inside the first instance

### Removed
- `general.monitor` config option — it was never wired up (yeet opens on the active monitor); existing configs with the key still parse

## [0.1.4] - 2026-02-22

### Security
- Fixed command injection vulnerability in app launching — `Exec` fields from `.desktop` files are no longer passed through a shell, preventing execution of metacharacters (`;`, `|`, `&`, etc.). Thanks to @Firstp1ck for responsible disclosure.
- Hardened history file handling

### Added
- `appearance.show_shortcuts` — toggle Alt+N shortcut badges (default: `true`)
- `appearance.show_descriptions` — toggle app description text (default: `true`)
- `appearance.row_height` — configurable row height
- `search.use_history` — toggle recent-launch prioritization in search results

### Fixed
- User CSS now loaded via `load_from_path` so `@import` directives resolve correctly (#4)
- Recent/history entries now correctly prioritized in search results (#2)
- Animation UX improvements (#1)

---

## [0.1.3] - 2025-12-17

### Added
- Scroll and touch support
- Nix flake
- Custom command entries documented in README

### Fixed
- Cargo.lock build issues

---

## [0.1.1] - 2025-12-17

### Added
- Initial public release
- GTK4 + Wayland layer shell app launcher
- Fuzzy search with launch history
- TOML config + CSS theming
- Catppuccin Macchiato default theme
