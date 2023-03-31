{
  description = "faerber - a tool to match your pictures to color palettes";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    pre-commit-hooks.url = "github:cachix/pre-commit-hooks.nix";
    pre-commit-hooks.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs = {
    self,
    nixpkgs,
    flake-utils,
    pre-commit-hooks,
  }:
    flake-utils.lib.eachDefaultSystem
    (
      system: let
        pkgs = import nixpkgs {
          inherit system;
        };
      in {
        devShells.default = let
          inherit (pkgs.stdenv.hostPlatform) isDarwin;
        in
          pkgs.mkShell {
            inherit (self.checks.${system}.pre-commit-check) shellHook;

            buildInputs = with pkgs;
              [
                cargo
                clippy
                libiconv
                openssl
                pkg-config
                rustc
                rustfmt
              ]
              ++ lib.optionals isDarwin [darwin.apple_sdk.frameworks.Security];
          };

        checks = {
          pre-commit-check = pre-commit-hooks.lib.${system}.run {
            src = ./.;
            hooks = {
              clippy.enable = true;
              rustfmt.enable = true;
            };
          };
        };

        packages = rec {
          faerber = pkgs.rustPlatform.buildRustPackage {
            name = "faerber";
            src = ./.;
            cargoSha256 = "sha256-i32he6OazvVvtn1O+w741lm2NDC+LyZb/qh7LF4/W0Y=";
          };
          default = faerber;
        };
      }
    );
}
