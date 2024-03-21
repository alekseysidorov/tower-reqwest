{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    treefmt-nix.url = "github:numtide/treefmt-nix";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay, treefmt-nix }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        # Setup nixpkgs
        pkgs = import nixpkgs {
          inherit system;

          overlays = [
            rust-overlay.overlays.default
            (final: prev: {
              rustToolchains = {
                stable = pkgs.rust-bin.stable.latest.default.override {
                  extensions = [
                    "rust-src"
                    "rust-analyzer"
                  ];
                };
                nightly = pkgs.rust-bin.nightly.latest.default;
              };
            })
          ];
        };

        # Eval the treefmt modules from ./treefmt.nix
        treefmt = (treefmt-nix.lib.evalModule pkgs ./treefmt.nix).config.build;
        # CI scripts
        ci = with pkgs; {
          tests = writeShellApplication {
            name = "ci-run-tests";
            runtimeInputs = [
              rustToolchains.stable
            ];
            text = ''
              cargo test --workspace --all-features --all-targets
              # TODO Add cargo publish test with the cargo workspaces
            '';
          };
          lints = writeShellApplication {
            name = "ci-run-lints";
            runtimeInputs = [
              rustToolchains.stable
            ];
            text = ''
              cargo clippy --workspace --all-features --all --all-targets
              cargo doc --workspace --all-features  --no-deps
            '';
          };
          # Run them all together
          all = writeShellApplication {
            name = "ci-run-all";
            runtimeInputs = [
              ci.lints
              ci.tests
            ];
            text = ''
              ci-run-tests
              ci-run-lints
            '';
          };
        };
      in
      {
        # for `nix fmt`
        formatter = treefmt.wrapper;
        # for `nix flake check`
        checks.formatting = treefmt.check self;

        devShells.default = pkgs.mkShell {
          nativeBuildInputs = with pkgs;
            [
              rustToolchains.stable
              ci.all
              ci.lints
              ci.tests
            ];
        };

        # Nightly compilator to run miri tests
        devShells.nightly = pkgs.mkShell {
          nativeBuildInputs = with pkgs; [
            rustToolchains.nightly
          ];
        };

        packages = {
          ci-lints = ci.lints;
          ci-tests = ci.tests;
          ci-all = ci.all;
        };
      });
}
