Name:           yeet-launcher
Version:        0.2.0
Release:        1%{?dist}
Summary:        Fast, minimal, configurable app launcher for Wayland

License:        GPL-3.0-only
URL:            https://github.com/1337hero/yeet
Source0:        %{url}/archive/refs/tags/v%{version}.tar.gz#/yeet-%{version}.tar.gz

BuildRequires:  cargo
BuildRequires:  rust
BuildRequires:  gtk4-devel
BuildRequires:  gtk4-layer-shell-devel

Provides:       yeet = %{version}

%description
A fast, minimal, configurable app launcher for Wayland compositors
(Hyprland, Sway, and other wlroots-based compositors, plus KDE Plasma).
Fuzzy search with launch-history ranking, TOML/CSS configuration, and a
dmenu mode for script-driven menus.

%prep
%autosetup -n yeet-%{version}

%build
cargo build --release --locked

%install
install -Dm755 target/release/yeet %{buildroot}%{_bindir}/yeet

%files
%license LICENSE
%doc README.md CHANGELOG.md
%{_bindir}/yeet

%changelog
* Wed Jul 08 2026 Mike Key <mike@mk3y.com> - 0.2.0-1
- Initial COPR release
