{
  description = "A stock exchange built on top of Kromer";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
    utils.url = "github:numtide/flake-utils/";

    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = {
    nixpkgs,
    utils,
    rust-overlay,
    ...
  }:
    utils.lib.eachDefaultSystem (system: let
      pkgs = import nixpkgs {
        inherit system;
        overlays = [(import rust-overlay)];
      };

      toolchain = pkgs.rust-bin.stable.latest;
    in {
      devShells.default = pkgs.mkShell {
        name = "kromer-api dev";

        packages = with pkgs; [
          toolchain.default
          cargo-deny
          sqlx-cli
          dbeaver-bin
        ];

        RUST_BACKTRACE = "full";
      };
    });
}
