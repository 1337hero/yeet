# Yeet - Development Commands

# Default: list available commands
default:
    @just --list

# Run in debug mode
run:
    cargo run

# Run in release mode
run-release:
    cargo run --release

# Build debug
build:
    cargo build

# Build optimized release
build-release:
    cargo build --release

# Bump version, commit, tag, and push
release version:
    sed -i 's/^version = ".*"/version = "{{version}}"/' Cargo.toml
    cargo check
    git add Cargo.toml
    git commit -m "Bump version to {{version}}"
    git tag v{{version}}
    git push origin main v{{version}}

# Run all checks (format, lint, test)
check:
    cargo fmt --check
    cargo clippy --all-targets -- -D warnings
    cargo test

# Format code
fmt:
    cargo fmt

# Lint with clippy
lint:
    cargo clippy --all-targets -- -D warnings

# Clean build artifacts
clean:
    cargo clean

# Install locally
install:
    cargo install --path .

# Install to system (requires sudo)
install-system:
    cargo build --release
    sudo cp target/release/yeet /usr/local/bin/

# Create user config directory with defaults
init-config:
    mkdir -p ~/.config/yeet
    @echo "Created ~/.config/yeet/"
    @echo "Copy defaults/config.toml and defaults/style.css there to customize"

# Watch for changes and rebuild
watch:
    cargo watch -x run

# Check binary size
size:
    cargo build --release
    @ls -lh target/release/yeet | awk '{print "Binary size:", $5}'

# Generate dependency tree
deps:
    cargo tree
