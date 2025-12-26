{
  description = "Hyprsession: A session saver for Hyprland";
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };
  outputs = {
    self,
    nixpkgs,
    flake-utils,
  }:
    flake-utils.lib.eachDefaultSystem (system: let
      pkgs = import nixpkgs {inherit system;};
      manifest = (pkgs.lib.importTOML ./Cargo.toml).package;
    in {
      packages = rec {
        default = pkgs.rustPlatform.buildRustPackage {
          pname = manifest.name;
          version = manifest.version;
          cargoLock.lockFile = ./Cargo.lock;
          src = pkgs.lib.cleanSource ./.;
        };
      };

      devShells = rec {
        default = rust-dev;
        
        rust-dev = pkgs.mkShell {
          nativeBuildInputs = with pkgs; [ 
            rustc 
            cargo 
            rust-analyzer
            clippy
            rustfmt
          ];
        };        
      };
    });
}
