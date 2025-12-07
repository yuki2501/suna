{
  description = "suna: sandbox-exec wrapper";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-24.05";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
        };

        rustPlatform = pkgs.rustPlatform;

        sunaPkg = rustPlatform.buildRustPackage {
          pname = "suna";
          version = "0.1.0";
          src = ./.;
          cargoLock.lockFile = ./Cargo.lock;
        };
      in {
        packages = {
          default = sunaPkg;
          suna = sunaPkg;
        };

        apps.default = flake-utils.lib.mkApp {
          drv = sunaPkg;
        };

        devShells.default = pkgs.mkShell {
          inputsFrom = [ sunaPkg ];

          buildInputs = with pkgs; [
            rustc
            cargo
            rustfmt
            clippy
            rust-analyzer
          ];
        };
      });
}
