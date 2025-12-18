# Summary
Open Yeet on the active monitor by default, while still allowing users to pin the launcher to a specific monitor via config.

## Problem
Yeet currently has a `general.monitor` config key documented in `defaults/config.toml` and `README.md`, but the UI code does not actually apply it. Additionally, users on multi-monitor setups want the launcher to appear on the monitor they’re actively using without having to hardcode a monitor index.

## Goal
- Make the launcher appear on the “active” monitor (best-effort) out of the box.
- Support an explicit monitor index for users who want a fixed monitor.
- Keep behavior Wayland-friendly and compositor-driven.

## Proposed Behavior
`general.monitor` semantics:
- If `monitor` is **unset**: open on the active monitor (best-effort detection; otherwise compositor-chosen).
- If `monitor = 0..`: pin the window to a specific monitor by index (based on GDK’s monitor list order).

Fallbacks:
- If monitor detection fails (or index is out of range), fall back to compositor-chosen placement.

## Implementation Notes (Wayland / layer-shell)
Yeet uses `gtk4-layer-shell` when supported. The layer-shell API supports selecting an output via:
- `LayerShell::set_monitor(Some(&gdk::Monitor))` to pin to an output
- `LayerShell::set_monitor(None)` to let the compositor choose

“Active monitor” is not universally exposed as a single stable concept via GTK/GDK. Best-effort options:
1. **Compositor-chosen**: use `set_monitor(None)` and let the compositor decide.
2. **Pointer-based heuristic**: read the pointer device’s surface at position and pick the monitor for that surface:
   - `display.default_seat()?.pointer()?.surface_at_position().0` → `display.monitor_at_surface(&surface)`

Option (2) is more deterministic for “active monitor” in most Hyprland/Sway setups, but should still gracefully fall back to (1).

## Config Changes
Change `Config.general.monitor` from `u32` → `Option<u32>` so “active by default” is representable without magic values.

Update docs:
- `defaults/config.toml`: remove `monitor` from embedded defaults (or comment it out) and document it as an optional override.
- `README.md`: update the example and explanation (show `monitor` as optional).

Backward compatibility:
- Existing user configs with `monitor = 0` continue to parse and behave the same (explicit pin).
- Consider also treating `monitor = 0` in older embedded defaults as “unset” if keeping the line around for now, to avoid changing behavior only after a defaults file update.

## Implementation Plan (Code)
1. **Config**
   - Update `src/config.rs`: change `GeneralConfig.monitor` to `Option<u32>` and default to `None`.
   - Update config merging so user config can override individual fields without resetting unspecified ones (deserialize user config into an `Overrides` struct with `Option<T>` fields and apply on top of the embedded defaults).
   - Update `defaults/config.toml` and `README.md` to reflect the new “optional override” behavior.
2. **Monitor selection helper**
   - Add a small helper in `src/ui.rs` (or a new `src/monitor.rs`) that:
     - Returns `Option<gdk::Monitor>` for a requested index
     - Returns `Option<gdk::Monitor>` for pointer-based “active” detection
3. **Apply in UI**
   - In `src/ui.rs` after `window.init_layer_shell()`:
     - If `config.general.monitor.is_some()`: resolve monitor by index and call `window.set_monitor(Some(&monitor))`
     - Else: try pointer-based monitor detection and set it; otherwise call `window.set_monitor(None)`
4. **Logging / UX**
   - If an explicit index is invalid, print a warning to stderr and fall back to compositor-chosen.
5. **Tests**
   - Unit test TOML parsing for `monitor = 0` and for “unset” monitor.
   - Unit test the “index in range/out of range” monitor-selection logic by extracting it into a pure function (no GTK dependency).

## Acceptance Criteria
- With no `monitor` override, Yeet opens on the active monitor (best-effort; at minimum compositor-chosen).
- Setting `monitor = N` pins Yeet to monitor `N` when layer-shell is available.
- Invalid values (e.g. `monitor = 99`) do not crash; Yeet falls back to compositor-chosen and emits a warning.
