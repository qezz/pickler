{
  description = "pickler env";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    nixpkgs-mozilla.url = "github:mozilla/nixpkgs-mozilla";
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
      nixpkgs-mozilla,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ (import nixpkgs-mozilla) ];
        };

        rusttoolchain =
          (pkgs.rustChannelOf {
            rustToolchain = ./rust-toolchain.toml;
            sha256 = "sha256-qqF33vNuAdU5vua96VKVIwuc43j4EFeEXbjQ6+l4mO4=";
          }).rust;
      in
      {
        devShells = {
          default = pkgs.mkShell {
            nativeBuildInputs = with pkgs; [
              rusttoolchain
              python315
            ];

            RUST_SRC_PATH = pkgs.rustPlatform.rustLibSrc;
          };
        };
      }
    );
}
