# Contributing to Yeet

Thanks for your interest in contributing! Yeet is a minimal app launcher — we'd like to keep it that way.

## Quick Start

```bash
# Clone and build
git clone https://github.com/1337hero/yeet.git
cd yeet

# Install GTK4 deps (Arch)
sudo pacman -S gtk4 gtk4-layer-shell

# Build and run
cargo build --release
cargo run --release
```

See [`AGENTS.md`](./AGENTS.md) for architecture details and code style guidelines.

## Before You Contribute

- **Search first**: Check if an issue already exists
- **Keep it minimal**: We prefer small, focused changes
- **Test it**: Run `just check` (or `cargo fmt --check && cargo clippy && cargo test`)

## Pull Requests

1. Fork and create a branch (`feature/thing` or `fix/thing`)
2. Make your changes
3. Run `just check` to verify formatting/lints/tests pass
4. Open a PR with a clear description

## Code Style

- `cargo fmt` before committing
- `cargo clippy` with no warnings
- See `AGENTS.md` for Rust conventions

## License

By contributing, you agree that your contributions will be licensed under GPL-3.0.
