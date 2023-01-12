{
  description = "The baz editor";

  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";

    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.flake-utils.follows = "flake-utils";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = {
    self,
    flake-utils,
    nixpkgs,
    rust-overlay,
  }:
    flake-utils.lib.eachDefaultSystem (system: let
      pkgs = import nixpkgs {
        inherit system;
        overlays = [
          (import rust-overlay)
        ];
      };

      rustToolchain = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
    in {
      devShells.default = pkgs.mkShell {
        buildInputs = with pkgs; [
          glib
          gtk3
          libsoup
          nodejs
          webkitgtk
        ];

        nativeBuildInputs = with pkgs;
          [
            cargo-deny
            nodePackages.npm
            nodePackages.svelte-language-server
            nodejs
            pkg-config
            rust-analyzer
            rustToolchain
          ]
          ++ pkgs.lib.optional (system == "x86_64-linux") pkgs.cargo-tarpaulin;
      };
    });
}
