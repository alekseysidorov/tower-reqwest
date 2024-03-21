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
  };

  packages = with pkgs; [
    nixpkgs-fmt
    dprint
    just
    cargo-nextest
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
