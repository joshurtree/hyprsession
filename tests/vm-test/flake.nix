{
  description = "Hyprsession Integration Test VM";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    home-manager = {
      url = "github:nix-community/home-manager";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, home-manager, ... }:
    let
      system = "x86_64-linux";
      pkgs = nixpkgs.legacyPackages.${system};
      
      # Build hyprsession from the parent directory
      hyprsession = pkgs.rustPlatform.buildRustPackage {
        pname = "hyprsession";
        version = "0.2.0";
        src = ../../.;
        cargoLock = {
          lockFile = ../../Cargo.lock;
        };
        nativeBuildInputs = with pkgs; [
          pkg-config
        ];
      };

      # VM configuration
      vmConfig = { config, pkgs, lib, ... }: {
        imports = [ 
          home-manager.nixosModules.home-manager {
              home-manager.useGlobalPkgs = true;
              home-manager.useUserPackages = true;
              home-manager.users.testuser = ./home.nix;
          }
        ];

        # Basic system configuration
        system.stateVersion = "25.11";
        
        # Enable Hyprland
        programs.hyprland = {
          enable = true;
          xwayland.enable = true;  
        };

        # Enable sudo without password for convenience
        security.sudo.wheelNeedsPassword = false;

        # System packages
        environment.systemPackages = with pkgs; [
          # Essential tools
          git
          curl
          wget
          vim
          htop
          jq
          
          # Hyprland ecosystem
          waybar
          rofi
          dunst
          
          # Terminal and basic apps
          kitty
          alacritty
          
          # Browsers for testing
          firefox
          
          # Additional GUI apps for testing
          gnome-calculator
          
          # Flatpak for testing
          flatpak
          
          # Our hyprsession binary
          hyprsession          
       ];

        # Flatpak setup
        services.flatpak.enable = true;
        
        # XDG portals for proper integration
        xdg.portal = {
          enable = true;
          wlr.enable = true;
          extraPortals = [ pkgs.xdg-desktop-portal-gtk ];
        };

        # Enable required services
        services.displayManager.sddm.enable = true;
        services.displayManager.sddm.wayland.enable = true;
        services.displayManager.autoLogin.enable = true;
        services.displayManager.autoLogin.user = "testuser";
        services.displayManager.defaultSession = "hyprland";
        
        # Networking
        networking = {
          hostName = "hyprsession-test";
          networkmanager.enable = true;
        };

        # Users
        users.users.testuser = {
          isNormalUser = true;
          password = "test";
          extraGroups = [ "wheel" "networkmanager" "audio" "video" ];
        };

        # Shared folder for test results
        # fileSystems."/shared" = {
        #   device = "shared";
        #   fsType = "9p";
        #   options = [ "trans=virtio" "version=9p2000.L" ];
        # };

        virtualisation.vmVariant = {
          services.qemuGuest.enable = true;
          virtualisation = {
            sharedDirectories.config = {
              source = "/home/josh/projects/hyprsession/tests/vm-test";
              target = "/shared";
              securityModel = "none";
            };

            memorySize = 4096;
            cores = 2;
            diskSize = 10000;
            qemu.options = [
              "-vga virtio"
              "-display gtk,gl=on"
            ];
          };
        };
      };
    in {
      # NixOS VM configuration
      nixosConfigurations.vm = nixpkgs.lib.nixosSystem {
        inherit system;
        modules = [ vmConfig ];
      };

      # Packages for easy access
      packages.${system} = {
        inherit hyprsession;
        
        vm = self.nixosConfigurations.vm.config.system.build.vm;
                
        # Just build the VM - let the shell script handle the rest
        run-test = self.packages.${system}.vm;
      };
        

      # Development shell for working with tests
      devShells.${system}.default = pkgs.mkShell {
        buildInputs = with pkgs; [
          hyprsession
          testScript
          qemu
          virt-viewer
        ];
        
        shellHook = ''
          echo "Hyprsession Test Environment"
          echo "=========================="
          echo ""
          echo "Available commands:"
          echo "  nix run .#run-test  - Build and run the test VM"
          echo "  nix build .#vm      - Build the VM image"
          echo "  nix run .#vm        - Run the built VM"
          echo ""
          echo "The test VM will:"
          echo "1. Start Hyprland with hyprsession"
          echo "2. Load test applications (Firefox, calculator, etc.)"
          echo "3. Save a session"
          echo "4. Clear the workspace"
          echo "5. Restore the session"
          echo "6. Compare before/after states"
          echo ""
          echo "Results will be saved in ./test-results/"
        '';
      };

      # Default package  
      defaultPackage.${system} = self.packages.${system}.run-test;
    };
}