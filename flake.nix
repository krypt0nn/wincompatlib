{
    inputs = {
        nixpkgs.url = "github:nixos/nixpkgs/nixos-24.05";
        nixpkgs-unstable.url = "github:nixos/nixpkgs/nixos-unstable";
        naersk.url = "github:nix-community/naersk";
        flake-utils.url = "github:numtide/flake-utils";
    };

    outputs = { self, flake-utils, ... }@inputs:
        flake-utils.lib.eachSystem [ "x86_64-linux" ] (system:
            let
                overlay-unstable = final: prev: {
                    unstable = inputs.nixpkgs-unstable.legacyPackages.${system};
                };

                pkgs = import inputs.nixpkgs {
                    inherit system;

                    overlays = [
                        overlay-unstable
                    ];
                };

                naersk' = pkgs.callPackage inputs.naersk {};

                fhs = pkgs.buildFHSUserEnv {
                    name = "wincompatlib-dev";

                    targetPkgs = pkgs: with pkgs; [
                        pkgs.unstable.rustup
                        pkgs.unstable.rustfmt
                        pkgs.unstable.clippy

                        gcc
                        cmake
                        pkg-config

                        # Needed for fonts installation
                        cabextract

                        self.defaultPackage
                    ];
                };

            in {
                defaultPackage = naersk'.buildPackage {
                    src = ./.;
                };

                devShells.default = fhs.env;
            }
        );
}
