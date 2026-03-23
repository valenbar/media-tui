{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      self,
      nixpkgs,
      utils,
    }:
    utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs { inherit system; };
        runtimeDependencies = with pkgs; [
          chafa
          dbus
          pkg-config
        ];
      in
      {

        defaultPackage = pkgs.rustPlatform.buildRustPackage {
          pname = "mpd-tui";
          version = "0.2.0";
          src = ./.;

          cargoHash = "sha256-cLS4shmI+i71rvH8kBLMalW5CqsqiXU/5Go/65NzWO4=";

          buildInputs = with pkgs; [ dbus ];

          nativeBuildInputs = with pkgs; [
            makeWrapper
            dbus
            pkg-config
          ];

          postInstall = ''
            wrapProgram $out/bin/mpd-tui \
              --prefix PATH : ${pkgs.lib.makeBinPath runtimeDependencies}
          '';

          meta.mainProgram = "mpd-tui";
        };

        devShell =
          with pkgs;
          mkShell {
            buildInputs = [
              cargo
              rustc
              rustfmt
              pre-commit
              rustPackages.clippy
              bacon
            ]
            ++ runtimeDependencies;

            RUST_SRC_PATH = rustPlatform.rustLibSrc;
          };
      }
    );
}
