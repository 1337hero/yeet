{
  description = "A fast, minimal app launcher for Wayland";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
      in
      {
        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = "yeet";
          version = "0.1.1";

          src = ./.;

          cargoLock = {
            lockFile = ./Cargo.lock;
          };

          nativeBuildInputs = with pkgs; [
            pkg-config
          ];

          buildInputs = with pkgs; [
            gtk4
            gtk4-layer-shell
          ];

          meta = with pkgs.lib; {
            description = "A fast, minimal app launcher for Wayland";
            homepage = "https://github.com/1337hero/yeet";
            license = licenses.gpl3Only;
            maintainers = [ ];
            platforms = platforms.linux;
            mainProgram = "yeet";
          };
        };

        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            cargo
            rustc
            rust-analyzer
            pkg-config
            gtk4
            gtk4-layer-shell
          ];
        };
      }
    );
}
