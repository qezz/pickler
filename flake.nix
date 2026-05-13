{
  description = "pickler env";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
      rust-overlay,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ rust-overlay.overlays.default ];
        };

        stable = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
        nightly = pkgs.rust-bin.nightly.latest.default;
      in
      {
        devShells = {
          default = pkgs.mkShell {
            nativeBuildInputs = with pkgs; [
              stable
              python315
            ];

            RUST_SRC_PATH = pkgs.rustPlatform.rustLibSrc;
          };

          fuzz = pkgs.mkShell {
            nativeBuildInputs = with pkgs; [
              nightly
              cargo-fuzz
              python315
            ];
          };
        };
      }
    );
}
