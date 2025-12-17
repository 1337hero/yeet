# Yeet

![License](https://img.shields.io/badge/license-GPL--3.0-blue)
![Rust](https://img.shields.io/badge/rust-1.85+-orange)

Hey 👋 I'm [@1337Hero](https://github.com/1337hero) and this is Yeet! A fast, minimal app launcher for Wayland. That's it.

## What Yeet Is

Yeet launches apps. That's it.

- **Fast** — Rust + GTK4, optimized release builds
- **Minimal** — Single binary, no daemons, no bloat
- **Smart search** — Substring-first with Skim fuzzy fallback (junk results filtered automatically)
- **Configurable** — TOML config + CSS theming
- **Wayland-native** — Layer shell overlay with keyboard grab

## What Yeet Isn't

Yeet is not trying to be an all-in-one tool. No clipboard manager, no calculator, no file browser, no emoji picker, no websearch, no plugins. If you want those, check out [walker](https://github.com/abenz1267/walker) or [wofi](https://hg.sr.ht/~scoopta/wofi).

## Screenshot

_Coming soon_

## Installation

### Arch Linux (AUR)

```sh
yay -S yeet-git
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
- `Up/Down` or `Ctrl+j/k` — Navigate results
- `Escape` — Close

Bind it to a key in your compositor (e.g., `Super+Space` in Hyprland/Sway).

## Configuration

Config lives in `~/.config/yeet/`. Yeet ships with sensible defaults — only override what you need.

### `config.toml`

```toml
[general]
max_results = 8
terminal = "alacritty"

[appearance]
width = 500
anchor_top = 200

[search]
min_score = 30        # absolute floor (used only when fuzzy fallback is active)
score_threshold = 0.6 # fuzzy fallback: keep matches within % of best score
prefer_prefix = true

[apps]
# Use display names, not .desktop filenames
favorites = ["Firefox", "Alacritty", "Visual Studio Code"]
exclude = ["Htop"]
```

### `style.css`

Full GTK4 CSS theming. Default theme is based on Catppuccin Macchiato with transparency for compositor blur.

```css
.yeet-window {
  background-color: alpha(#1e1e2e, 0.9);
  border-radius: 12px;
}

.yeet-entry {
  font-size: 24px;
  padding: 16px;
}
```

**CSS classes:** `.yeet-window`, `.yeet-container`, `.yeet-entry`, `.yeet-list`, `.yeet-row`, `.yeet-icon`, `.yeet-app-name`, `.yeet-app-desc`, `.yeet-shortcut`

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
