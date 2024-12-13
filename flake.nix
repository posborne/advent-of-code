{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };
        my-rust = pkgs.rust-bin.stable.latest.complete.override {
          extensions = ["rust-src"];
          targets = ["wasm32-wasip1"];
        };
      in
      {
        devShell = with pkgs; mkShell {
          buildInputs = [
            bacon
            cargo-nextest
            my-rust
            wasmtime
          ];
          RUST_SRC_PATH = rustPlatform.rustLibSrc;
          CARGO_TARGET_WASM32_UNKNOWN_UNKNOWN_LINKER = "wasm-ld";
        };
      }
    );
}
