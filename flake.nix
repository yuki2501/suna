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

        # このリポジトリ自体をビルド対象にする
        sunaPkg = rustPlatform.buildRustPackage {
          pname = "suna";
          version = "0.1.0";
          # flake のルートが Rust プロジェクトのルートという前提
          src = ./.;
          # Cargo.lock を使って依存を固定
          cargoLock.lockFile = ./Cargo.lock;
        };
      in {
        # nix build .#suna とか nix build .#default でビルドできるようにする
        packages = {
          default = sunaPkg;
          suna = sunaPkg;
        };

        # nix run . で実行できるように app も定義しておく
        apps.default = flake-utils.lib.mkApp {
          drv = sunaPkg;
        };

        # いままでの devShell も残す。開発中は nix develop で入る。
        devShells.default = pkgs.mkShell {
          # ビルド時と同じ依存を共有しておくと微妙に楽
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
