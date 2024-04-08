{
  description = "Brongan's attempt at signal";
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    crane = {
      url = "github:ipetkov/crane";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs = {
        nixpkgs.follows = "nixpkgs";
        flake-utils.follows = "flake-utils";
      };
    };
  };
  outputs = {  nixpkgs, crane, flake-utils, rust-overlay, ... }:
    flake-utils.lib.eachSystem [ "x86_64-linux" ] (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          config.allowUnfree = true;
          overlays = [ (import rust-overlay) ];
        };
        sqliteStatic = pkgs.pkgsMusl.sqlite.override {
          stdenv =
            pkgs.pkgsStatic.stdenv;
        };
        inherit (pkgs) lib;
        nativeToolchain = pkgs.rust-bin.nightly.latest.default.override {
          targets = [ "x86_64-unknown-linux-musl" ];
        };
        nativeCraneLib = (crane.mkLib pkgs).overrideToolchain nativeToolchain;
        src = lib.cleanSource ./.;
        commonArgs = {
          inherit src;
          version = "0.1.0";
          strictDeps = true;
          nativeBuildInputs = with pkgs; [ pkg-config ];
        };
        nativeArgs = commonArgs // {
          CARGO_BUILD_TARGET = "x86_64-unknown-linux-musl";
          CARGO_BUILD_RUSTFLAGS = "-C target-feature=+crt-static";
          buildInputs = [ sqliteStatic ];
        };
        nativeArtifacts = nativeCraneLib.buildDepsOnly nativeArgs;
        myServer = nativeCraneLib.buildPackage (nativeArgs // {
		  pname = "brongnal-server";
          cargoArtifacts = nativeArtifacts;
        });
        dockerImage = pkgs.dockerTools.streamLayeredImage {
          name = "brongnal";
          tag = "latest";
          contents = [ myServer ];
          config = {
            Cmd = [
              "${myServer}/bin/server"
            ];
            Env = ["RUST_LOG=info"];
          };
        };
      in
      {
        packages = {
          inherit myServer dockerImage;
          default = myServer;
        };
      }
    );
}
