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

  outputs =
    { self
    , nixpkgs
    , flake-utils
    , rust-overlay
    , treefmt-nix
    }: flake-utils.lib.eachDefaultSystem (system:
    let
      # Setup nixpkgs
      pkgs = import nixpkgs {
        inherit system;

        overlays = [
          rust-overlay.overlays.default
          (final: prev: {
            rustToolchains = {
              stable = prev.rust-bin.stable.latest.default.override {
                extensions = [
                  "rust-src"
                  "rust-analyzer"
                ];
              };
              nightly = prev.rust-bin.nightly.latest.default;
            };
          })
        ];
      };
      # Setup runtime dependencies
      runtimeInputs = with pkgs; [
        rustToolchains.stable
        cargo-nextest
        openssl.dev
        pkg-config
      ]
      # Some additional libraries for the Darwin platform
      ++ lib.optionals stdenv.isDarwin [
        darwin.apple_sdk.frameworks.SystemConfiguration
      ];

      # Eval the treefmt modules from ./treefmt.nix
      treefmt = (treefmt-nix.lib.evalModule pkgs ./treefmt.nix).config.build;
      # CI scripts
      ci = with pkgs; {
        tests = writeShellApplication {
          name = "ci-run-tests";
          inherit runtimeInputs;
          text = ''
            cargo nextest run --all-features --workspace --all-targets
            cargo test --workspace --all-features --doc
          '';
        };

        lints = writeShellApplication {
          name = "ci-run-lints";
          inherit runtimeInputs;
          text = ''
            cargo clippy --workspace --all-features --all --all-targets
            cargo doc --workspace --all-features  --no-deps
          '';
        };

        semver_checks = writeShellApplication {
          name = "ci-run-semver-checks";
          runtimeInputs = with pkgs; [ cargo-semver-checks ];
          text = ''cargo semver-checks'';
        };

        # Run them all together
        all = writeShellApplication {
          name = "ci-run-all";
          runtimeInputs = [ ci.lints ci.tests ];
          text = ''
            ci-run-tests
            ci-run-lints
            ci-run-semver-checks
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
        nativeBuildInputs = runtimeInputs ++ [
          ci.all
          ci.lints
          ci.tests
          ci.semver_checks
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
        ci-semver-checks = ci.semver_checks;
        ci-all = ci.all;
      };
    });
}
