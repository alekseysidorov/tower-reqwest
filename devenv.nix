{ pkgs, ... }:
{
  languages = {
    nix.enable = true;
    rust = {
      enable = true;
      # https://devenv.sh/reference/options/#languagesrustchannel
      channel = "stable";
      components = [ "rustc" "cargo" "clippy" "rustfmt" "rust-analyzer" ];
    };
    # Explicitly disable C language support to reduce the amount of dependencies installed by devshell.
    c.enable = false;
    cplusplus.enable = false;
  };

  packages = with pkgs; [
    nixpkgs-fmt
    dprint
    just
    cargo-nextest
    # reqwest dependencies
    openssl.dev
    pkg-config
  ];

  scripts = {
    ci-fmt.exec = "dprint fmt";
    ci-lints.exec = ''
      cargo clippy --workspace --all-features --all --all-targets
      cargo doc --workspace --all-features  --no-deps
    '';
    ci-tests.exec = "cargo nextest run --workspace";
  };

  enterTest = ''
    ci-tests
  '';
}
