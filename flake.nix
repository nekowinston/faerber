{
  description = "faerber - a tool to match your pictures to color palettes";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-parts.url = "github:hercules-ci/flake-parts";

    crane.url = "github:ipetkov/crane";
    crane.inputs.nixpkgs.follows = "nixpkgs";
    rust-overlay.url = "github:oxalica/rust-overlay";

    pre-commit-hooks.url = "github:cachix/pre-commit-hooks.nix";
    pre-commit-hooks.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs = inputs @ {flake-parts, ...}:
    flake-parts.lib.mkFlake {inherit inputs;} {
      systems = ["x86_64-linux"];
      perSystem = {
        config,
        system,
        pkgs,
        self',
        ...
      }: let
        inherit (pkgs) lib;
        inherit (pkgs.stdenv.hostPlatform) isDarwin;
        rust-toolchain = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
        craneLib = (inputs.crane.mkLib pkgs).overrideToolchain rust-toolchain;
      in rec {
        _module.args.pkgs = import inputs.nixpkgs {
          inherit system;
          overlays = [(import inputs.rust-overlay)];
        };

        devShells.default = config.pre-commit.devShell.overrideAttrs (_: {
          buildInputs =
            lib.flatten (lib.mapAttrsToList (name: pkg: pkg.buildInputs) self'.packages)
            ++ (with pkgs; [
              rust-toolchain
              # WASM dependencies, needed for the npm release only
              binaryen
              wasm-bindgen-cli
              wasm-pack
            ]);
        });

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

        legacyPackages =
          self'.packages
          // {
            containers = {
              discord-bot = pkgs.dockerTools.buildLayeredImage {
                name = "faerber-discord-bot";
                tag = "latest";
                created = let
                  d = s: e: lib.substring s e inputs.self.lastModifiedDate;
                in "${d 0 4}-${d 4 2}-${d 6 2}T${d 8 2}:${d 10 2}:${d 12 2}Z";
                contents = [
                  packages.discord-bot
                  pkgs.tini
                ];
                config.Entrypoint = [
                  "${pkgs.tini}/bin/tini"
                  "${packages.discord-bot}/bin/discord-bot"
                ];
              };
            };
          };

        pre-commit = {
          check.enable = true;
          settings.hooks = {
            clippy.enable = true;
            rustfmt.enable = true;
            taplo.enable = true;
          };
        };
      };

      imports = [inputs.pre-commit-hooks.flakeModule];
    };
}
