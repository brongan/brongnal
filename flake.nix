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
        toolchain = pkgs.rust-bin.nightly.latest.default.override {
          targets = [ "x86_64-unknown-linux-musl" ];
        };
        craneLib = (crane.mkLib pkgs).overrideToolchain toolchain;
        src = lib.cleanSource ./.;
        args = {
          inherit src;
          version = "0.1.0";
          strictDeps = true;
          nativeBuildInputs = with pkgs; [ pkg-config protobuf ];
		  CARGO_BUILD_TARGET = "x86_64-unknown-linux-musl";
          CARGO_BUILD_RUSTFLAGS = "-C target-feature=+crt-static";
          buildInputs = [ sqliteStatic ];
		  pname = "server";
        };
        nativeArtifacts = craneLib.buildDepsOnly args;
        myServer = craneLib.buildPackage (args // {
		  cargoExtraArgs = "--package=server";
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
            Env = ["RUST_LOG=debug"];
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
