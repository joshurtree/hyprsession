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

        # C++ version
        hyprsession-cpp = pkgs.stdenv.mkDerivation {
          pname = "${manifest.name}-cpp";
          version = manifest.version;
          src = pkgs.lib.cleanSource ./.;
          
          nativeBuildInputs = with pkgs; [
            cmake
            pkg-config
          ];
          
          buildInputs = with pkgs; [
            nlohmann_json
            wayland
            hyprland
          ];
          
          buildPhase = ''
            mkdir -p build
            cd build
            cmake .. -DCMAKE_BUILD_TYPE=Release
            make -j$NIX_BUILD_CORES
          '';
          
          installPhase = ''
            mkdir -p $out/bin
            cp hyprsession-cpp $out/bin/
          '';
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
        
        cpp-dev = pkgs.mkShell {
          nativeBuildInputs = with pkgs; [
            cmake
            gcc
            gdb
            pkg-config
            clang-tools
          ];
          
          buildInputs = with pkgs; [
            nlohmann_json
            wayland
            hyprland
          ];
          
          shellHook = ''
            echo "C++ Development Environment"
            echo "Available tools: cmake, g++, gdb, pkg-config"
            echo "To build: mkdir -p build_cpp && cd build_cpp && cmake .. && make"
          '';
        };
      };
    });
}
