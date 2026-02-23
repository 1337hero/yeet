# Plan: Fix Three GitHub Issues (#1, #2, #4)

## Task Description
Address three open GitHub issues for the Yeet app launcher:
1. **Issue #1** — Jarring expand/contract animation when result count changes during search
2. **Issue #2** — Add recent search priority (track launched apps, boost their scores)
3. **Issue #4** — CSS `@import` doesn't work in user style.css

All work happens on `main` branch (or a new combined branch). The existing `feat/prioritize-recently-launched-apps` branch has cleanup work (simplified `clean_exec`, removed shlex) that's already merged into current `main` — the actual feature isn't implemented there.

## Objective
Close all three issues with working implementations, passing `cargo clippy` and `cargo test`, ready for release.

## Problem Statement
1. **Animation UX (#1)**: When typing/backspacing changes result count, the window resizes with visible expansion/contraction. CSS `transition: none` on list elements doesn't help because GTK4 auto-sizes the `ApplicationWindow` based on child content. The window itself animates its size allocation.
2. **Recent Priority (#2)**: Results are alphabetical (with favorites boost). Users want recently launched apps prioritized — type "cur" and Cursor appears first if you launched it recently.
3. **CSS @import (#4)**: `load_css()` reads user CSS as a string via `read_to_string` then calls `provider.load_from_data()`. This gives GTK no file context to resolve `@import` paths. Need to use `load_from_path()` or `load_from_file()` for user CSS.

## Solution Approach

### Issue #1: Fixed-height results list
- Set the `ListBox` to a fixed height based on `max_results * row_height` so the window never resizes when result count changes.
- Use `list_box.set_size_request(-1, fixed_height)` to lock the vertical space.
- Row height = ~56px (8px top margin + 8px bottom margin + ~36px icon + padding). Calculate from config: `max_results * row_height`.
- Add `row_height` to `AppearanceConfig` with a sensible default (56).

### Issue #2: Launch history tracking + score boost
- On `launch_app()`, append the app name + timestamp to `~/.local/share/yeet/history.txt` (one line per launch: `timestamp\tapp_name`).
- On startup, load history file, build a recency map: `HashMap<String, u64>` (app name → most recent timestamp).
- In search scoring, add a recency boost to apps that appear in history. Boost decays over time (e.g., launched today = +100, yesterday = +50, older = +20).
- Cap history file at 200 lines (trim oldest on write).
- Add `[search] use_history = true` config option (default true).

### Issue #4: Use `load_from_path` for user CSS
- When user CSS file exists, use `provider.load_from_path(&user_path)` instead of reading to string + `load_from_data`. This lets GTK resolve `@import` relative to the file's directory.
- Keep `load_from_data` only for the embedded default CSS (which has no imports).

## Relevant Files
Use these files to complete the task:

- `src/ui.rs` — Search filtering logic (issue #1 fixed height, #2 score boost integration), CSS loading (#4), `populate_list`, `build_ui`
- `src/desktop.rs` — `launch_app()` function where history write happens (#2), `App` struct
- `src/config.rs` — Add `row_height` to `AppearanceConfig` (#1), add `use_history` to `SearchConfig` (#2)
- `defaults/config.toml` — Add new config keys with defaults and documentation
- `defaults/style.css` — May need minor CSS adjustments for fixed height
- `Cargo.toml` — No new dependencies needed (use `std::time` for timestamps, `std::fs` for history file)

### New Files
- `src/history.rs` — New module for launch history read/write/trim logic (#2)

## Implementation Phases

### Phase 1: Foundation
- Create `src/history.rs` with history file read/write/trim
- Add new config fields (`row_height`, `use_history`)
- Update `defaults/config.toml`

### Phase 2: Core Implementation
- Fix CSS loading to use `load_from_path` for user files (#4)
- Implement fixed-height list in `build_ui` (#1)
- Wire history tracking into `launch_app` and search scoring (#2)

### Phase 3: Integration & Polish
- Run `cargo fmt`, `cargo clippy --all-targets -- -D warnings`, `cargo test`
- Test build with `cargo build --release`
- Verify all three fixes work as expected

## Team Orchestration

- You operate as the team lead and orchestrate the team to execute the plan.
- You NEVER write code directly — deploy builders and validators via Task tools.
- Take note of session IDs for resume capability.

### Team Members

- Builder
  - Name: builder-css-import
  - Role: Fix CSS @import support (Issue #4) — smallest, most isolated change
  - Agent Type: general-purpose
  - Resume: true

- Builder
  - Name: builder-fixed-height
  - Role: Implement fixed-height results list (Issue #1) — config + UI change
  - Agent Type: general-purpose
  - Resume: true

- Builder
  - Name: builder-history
  - Role: Implement launch history tracking and search boost (Issue #2) — new module + integration
  - Agent Type: general-purpose
  - Resume: true

- Builder
  - Name: validator
  - Role: Run clippy, tests, build; verify all three fixes
  - Agent Type: validator
  - Resume: false

## Step by Step Tasks

### 1. Fix CSS @import Support (Issue #4)
- **Task ID**: fix-css-import
- **Depends On**: none
- **Assigned To**: builder-css-import
- **Agent Type**: general-purpose
- **Parallel**: true
- In `src/ui.rs` `load_css()`: when loading from user file path, use `provider.load_from_path(&user_path)` instead of `read_to_string` + `load_from_data`
- Keep `load_from_data(DEFAULT_STYLE)` for the embedded default (no file path to resolve imports against)
- The function should become:
  ```rust
  fn load_css() {
      let provider = CssProvider::new();

      if let Some(user_path) = Config::user_style_path() {
          if user_path.exists() {
              provider.load_from_path(&user_path);
          } else {
              provider.load_from_data(DEFAULT_STYLE);
          }
      } else {
          provider.load_from_data(DEFAULT_STYLE);
      }

      gtk4::style_context_add_provider_for_display(
          &Display::default().expect("Could not get default display"),
          &provider,
          gtk4::STYLE_PROVIDER_PRIORITY_USER,
      );
  }
  ```
- Verify `load_from_path` accepts a `&Path` — check GTK4-rs API. It may need `load_from_file` with a `gio::File` instead.

### 2. Add Config Fields
- **Task ID**: add-config-fields
- **Depends On**: none
- **Assigned To**: builder-fixed-height
- **Agent Type**: general-purpose
- **Parallel**: true (can run alongside task 1)
- In `src/config.rs`:
  - Add `row_height: i32` to `AppearanceConfig` with default `56`
  - Add `use_history: bool` to `SearchConfig` with default `true`
  - Add corresponding default functions
  - Update `Default` impl for both structs
- In `defaults/config.toml`:
  - Add `row_height = 56` under `[appearance]` with comment
  - Add `use_history = true` under `[search]` with comment
- Add unit tests for the new config fields parsing

### 3. Implement Fixed-Height Results List (Issue #1)
- **Task ID**: fix-animation
- **Depends On**: add-config-fields
- **Assigned To**: builder-fixed-height
- **Agent Type**: general-purpose
- **Parallel**: false
- In `src/ui.rs` `build_ui()`, after creating `list_box`:
  - Calculate fixed height: `config.appearance.row_height * config.general.max_results as i32`
  - Call `list_box.set_size_request(-1, fixed_height)` to lock vertical space
- This prevents the window from resizing when result count changes
- The list area stays constant size; fewer results just leave empty space at the bottom

### 4. Create History Module (Issue #2)
- **Task ID**: create-history-module
- **Depends On**: add-config-fields
- **Assigned To**: builder-history
- **Agent Type**: general-purpose
- **Parallel**: true (can run alongside task 3)
- Create `src/history.rs` with:
  - `history_path() -> PathBuf` — returns `~/.local/share/yeet/history.txt`
  - `record_launch(app_name: &str)` — append `{unix_timestamp}\t{app_name}\n` to history file (create dirs if needed)
  - `load_history() -> HashMap<String, u64>` — read file, parse lines, keep most recent timestamp per app name
  - `trim_history(max_lines: usize)` — if file exceeds max, keep only the most recent `max_lines` entries
  - Constants: `MAX_HISTORY_LINES = 200`
- Add `mod history;` to `main.rs`
- Write unit tests for parsing and trimming logic

### 5. Wire History Into Launch and Search
- **Task ID**: wire-history
- **Depends On**: create-history-module
- **Assigned To**: builder-history
- **Agent Type**: general-purpose
- **Parallel**: false
- In `src/desktop.rs` `launch_app()`: call `history::record_launch(&app.name)` after successful spawn
- In `src/ui.rs` `build_ui()`:
  - Load history at startup: `let history = if use_history { history::load_history() } else { HashMap::new() }`
  - Pass history into the search closure
  - In the scoring logic, add recency boost:
    ```rust
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let recency_boost = history.get(&apps[i].name)
        .map(|&last| {
            let age_hours = (now.saturating_sub(last)) / 3600;
            match age_hours {
                0..=24 => 100,    // last 24h
                25..=168 => 50,   // last week
                _ => 20,          // older
            }
        })
        .unwrap_or(0);
    let final_score = score + recency_boost;
    ```
  - Apply boost in both substring-match and fuzzy-match branches
  - Also apply to initial results (empty query) — sort by recency if history exists

### 6. Validate All Changes
- **Task ID**: validate-all
- **Depends On**: fix-css-import, fix-animation, wire-history
- **Assigned To**: validator
- **Agent Type**: validator
- **Parallel**: false
- Run `cargo fmt --check`
- Run `cargo clippy --all-targets -- -D warnings`
- Run `cargo test`
- Run `cargo build --release`
- Review that all three issues are addressed
- Verify no regressions in existing tests

## Acceptance Criteria
- [ ] **Issue #1**: Results list has fixed height; window does not resize when result count changes during typing/backspacing
- [ ] **Issue #2**: Launching an app records it to history file; recently launched apps appear higher in search results; history caps at 200 entries
- [ ] **Issue #4**: User CSS with `@import 'colors.css'` works when colors.css is in `~/.config/yeet/`
- [ ] `cargo fmt --check` passes
- [ ] `cargo clippy --all-targets -- -D warnings` passes
- [ ] `cargo test` passes (including new tests for config fields and history module)
- [ ] `cargo build --release` succeeds

## Validation Commands
Execute these commands to validate the task is complete:

- `cargo fmt --check` — Verify formatting
- `cargo clippy --all-targets -- -D warnings` — Lint check
- `cargo test` — Run all tests
- `cargo build --release` — Verify release build
- `ls ~/.local/share/yeet/` — Verify history directory created on first launch (manual)

## Notes
- No new crate dependencies needed. `std::time`, `std::fs`, `std::io` cover history needs.
- The `feat/prioritize-recently-launched-apps` branch diff shows it simplified `desktop.rs` (removed shlex, simplified exec handling). That cleanup is already in `main` based on the current source. The branch can be deleted after #2 is implemented on main.
- GTK4-rs `CssProvider::load_from_path` may need a string path or `&Path` — check the API. If it requires `gio::File`, use `provider.load_from_file(&gio::File::for_path(&user_path))`.
- The fixed-height approach for #1 is the simplest fix. Alternative: wrap ListBox in a ScrolledWindow with fixed height — but that adds scroll complexity we don't need since max_results already caps the list.
- History boost values (100/50/20) are hardcoded initially. Could be config later if users want to tune, but not now — keep it simple.
