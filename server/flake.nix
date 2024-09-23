{
  description = "jacobhenn's Rust dev flake";

  inputs = {
    nixpkgs.url      = "github:NixOS/nixpkgs/nixos-unstable";
    surreal.url      = "github:surrealdb/surrealdb/v2.0.1";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url  = "github:numtide/flake-utils";

  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils, ... }:
  
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
          config.allowUnfree = true;
        };
      in
      with pkgs;
      {
        devShells.default = mkShell {
          name = "rust-dev";
          buildInputs = [
            pkg-config
            (
              rust-bin.selectLatestNightlyWith (toolchain: toolchain.default.override {
                extensions = [ "rust-src" "rust-analyzer" ];
                targets = [ "x86_64-unknown-linux-gnu" ];
              })
            )
            surrealdb
          ];
        };
      }
    );
}
