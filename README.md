# Yeet

![License](https://img.shields.io/badge/license-GPL--3.0-blue)
![Rust](https://img.shields.io/badge/rust-1.85+-orange)

Hey 👋 I'm [@1337Hero](https://github.com/1337hero) and this is Yeet! A fast, minimal app launcher for Wayland. That's it.

![Yeet!](yeet.png)

## What Yeet Is

Yeet launches apps. That's it.

- **Fast** — Rust + GTK4, optimized release builds
- **Minimal** — Single binary, no daemons, no bloat
- **Smart search** — Substring-first with Skim fuzzy fallback (junk results filtered automatically)
- **Configurable** — TOML config + CSS theming
- **Wayland-native** — Layer shell overlay with keyboard grab

## What Yeet Isn't

Yeet is not trying to be an all-in-one tool. No clipboard manager, no calculator, no file browser, no emoji picker, no websearch, no plugins. If you want those, check out [walker](https://github.com/abenz1267/walker) or [wofi](https://hg.sr.ht/~scoopta/wofi).

## Installation

### Arch Linux (AUR)

```sh
yay -S yeet-git
```

### Nix

```sh
# Run directly
nix run github:1337hero/yeet

# Install to profile
nix profile install github:1337hero/yeet
```

### From Source

```sh
git clone https://github.com/1337hero/yeet
cd yeet
cargo build --release
sudo cp target/release/yeet /usr/local/bin/
```

**Dependencies:** GTK4, gtk4-layer-shell

## Usage

```sh
yeet
```

- Type to search
- `Enter` — Launch selected app
- `Up/Down` — Navigate results
- `Scroll` / `Trackpad` — Navigate results
- `Alt+1-9` — Quick launch by position
- `Escape` — Close

Bind it to a key in your compositor (e.g., `Super+Space` in Hyprland/Sway).

## Configuration

Config lives in `~/.config/yeet/`. Yeet ships with sensible defaults — only override what you need.

### `config.toml`

```toml
[general]
monitor = 0           # Show on specific monitor (0 = primary)
max_results = 8       # Max results when searching
initial_results = 8   # Results shown before typing (0 = just search bar)
terminal = "alacritty"

[appearance]
width = 500           # Window width (height auto-sizes)
anchor_top = 200      # Distance from top of screen

[search]
min_score = 30        # Absolute floor for fuzzy fallback
score_threshold = 0.6 # Keep matches within % of best score (0.0-1.0)
prefer_prefix = true  # Prioritize exact prefix matches

[apps]
extra_dirs = []       # Additional directories to scan for .desktop files
exclude = ["Htop"]    # Apps to hide (use display names)
favorites = ["Firefox", "Alacritty"]  # Pin to top (use display names)

# Custom app entries
[[apps.custom]]
name = "My Script"
exec = "/path/to/script.sh"
icon = "utilities-terminal"  # optional, from icon theme
keywords = ["alias", "shortcut"]  # optional, extra search terms
```

### `style.css`

Full GTK4 CSS theming. Copy `defaults/style.css` to `~/.config/yeet/style.css` and customize. Default theme is Catppuccin Macchiato with transparency for compositor blur.

```css
/* Use alpha() for compositor blur (Hyprland/Sway) */
.yeet-window {
  background-color: alpha(#1e1e2e, 0.9);
  border-radius: 12px;
}

.yeet-entry {
  font-size: 24px;
  padding: 16px;
  caret-color: #a6e3a1;
}

.yeet-row:selected {
  background-color: #45475a;
}
```

**CSS classes:**
| Class | Element |
|-------|---------|
| `.yeet-window` | Main window |
| `.yeet-container` | Inner container |
| `.yeet-entry` | Search input |
| `.yeet-list` | Results list |
| `.yeet-row` | Result row (supports `:selected`, `:hover`) |
| `.yeet-row-content` | Row inner content |
| `.yeet-icon` | App icon |
| `.yeet-app-name` | App name label |
| `.yeet-app-desc` | App description |
| `.yeet-shortcut` | Alt+N shortcut badge |

## Building

Requires Rust 1.85+ and GTK4 development libraries.

```sh
# Dev build
cargo run

# Release build
cargo build --release

# Format + lint (mirrors CI)
cargo fmt && cargo clippy --all-targets -- -D warnings
```

## Why "Yeet"?

You yeet apps into existence.

## License

GPL-3.0
