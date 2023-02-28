{
  description = "faerber - a tool to match your pictures to color palettes";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = {
    self,
    nixpkgs,
    flake-utils,
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
        packages = rec {
          faerber = pkgs.rustPlatform.buildRustPackage {
            name = "faerber";
            src = ./.;
            cargoSha256 = "sha256-HNH2z22sJwJNevbejVDQq9DzG/aPoEJta9LenbKq/H4=";
          };
          default = faerber;
        };
      }
    );
}
