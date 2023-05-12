{
  description = "faerber - a tool to match your pictures to color palettes";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";

    crane.url = "github:ipetkov/crane";
    crane.inputs.nixpkgs.follows = "nixpkgs";
    rust-overlay.url = "github:oxalica/rust-overlay";

    pre-commit-hooks.url = "github:cachix/pre-commit-hooks.nix";
    pre-commit-hooks.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs = {
    self,
    nixpkgs,
    flake-utils,
    crane,
    pre-commit-hooks,
    rust-overlay,
  }:
    flake-utils.lib.eachDefaultSystem
    (
      system: let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [(import rust-overlay)];
        };
        inherit (pkgs) lib;
        inherit (pkgs.stdenv.hostPlatform) isDarwin;
        rust-toolchain = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
        craneLib = (crane.mkLib pkgs).overrideToolchain rust-toolchain;
      in {
        devShells.default = pkgs.mkShell {
          buildInputs =
            lib.flatten (lib.mapAttrsToList (name: pkg: pkg.buildInputs) self.packages.${system})
            ++ (with pkgs; [
              rust-toolchain
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
          src = let
            custom = path: _type: builtins.match "^.*/vendor/.*$" path != null;
            filter = path: type: (custom path type) || (craneLib.filterCargoSources path type);
          in
            lib.cleanSourceWith {
              src = craneLib.path ./.;
              inherit filter;
            };
        in rec {
          faerber = craneLib.buildPackage rec {
            name = "faerber";
            pname = name;

            inherit src;
            cargoExtraArgs = "-p ${name}";

            buildInputs = with pkgs; [
              libiconv
              openssl
              pkg-config
              zlib
            ];
          };
          faerber-app = craneLib.buildPackage rec {
            name = "faerber-app";
            pname = name;

            inherit src;
            cargoExtraArgs = "-p ${name}";

            buildInputs =
              faerber.buildInputs
              ++ lib.optionals isDarwin (with pkgs.darwin.apple_sdk_11_0.frameworks; [
                # GUI dependencies on darwin
                AppKit
              ]);
          };
          discord-bot = craneLib.buildPackage rec {
            name = "discord-bot";
            pname = name;

            inherit (faerber) src;
            buildInputs =
              faerber.buildInputs
              ++ lib.optionals isDarwin (with pkgs.darwin.apple_sdk_11_0.frameworks; [
                # GUI dependencies on darwin
                Security
              ]);
            cargoExtraArgs = "-p ${name}";
          };
          default = faerber;
        };
      }
    );
}
