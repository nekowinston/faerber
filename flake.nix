{
  description = "faerber - a tool to match your pictures to color palettes";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
    pre-commit-hooks.url = "github:cachix/pre-commit-hooks.nix";
    pre-commit-hooks.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs = {
    self,
    nixpkgs,
    flake-utils,
    pre-commit-hooks,
    rust-overlay,
  }:
    flake-utils.lib.eachDefaultSystem
    (
      system: let
        overlays = [(import rust-overlay)];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
        inherit (pkgs) lib;
        inherit (pkgs.stdenv.hostPlatform) isDarwin;
      in {
        devShells.default = pkgs.mkShell {
          buildInputs =
            lib.flatten (lib.mapAttrsToList (name: pkg: pkg.buildInputs) self.packages.${system})
            ++ (with pkgs; [
              # WASM dependencies, needed for the npm release only
              binaryen
              wasm-bindgen-cli
              wasm-pack
            ]);
          shellHook = ''
            ${self.checks.${system}.pre-commit-check.shellHook}
          '';
        };

        checks = {
          pre-commit-check = pre-commit-hooks.lib.${system}.run {
            src = ./.;
            hooks = {
              rustfmt.enable = true;
            };
          };
        };

        packages = let
          src = lib.cleanSource ./.;
          cargoLock = {
            lockFile = ./Cargo.lock;
          };
        in rec {
          faerber = pkgs.rustPlatform.buildRustPackage {
            name = "faerber";
            inherit src cargoLock;

            buildInputs = with pkgs; [
              (rust-bin.fromRustupToolchainFile ./rust-toolchain.toml)
              libiconv
              openssl
              pkg-config
              zlib
            ];
          };
          faerber-app = pkgs.rustPlatform.buildRustPackage {
            name = "faerber-app";
            inherit src cargoLock;

            buildAndTestSubdir = "app";
            buildInputs =
              faerber.buildInputs
              ++ lib.optionals isDarwin (with pkgs.darwin.apple_sdk_11_0.frameworks; [
                # GUI dependencies on darwin
                AppKit
                CoreServices
                OpenGL
                Security
              ]);
          };
          default = faerber;
        };
      }
    );
}
