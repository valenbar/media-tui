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
          pname = "media-tui";
          version = "0.2.0";
          src = ./.;

          cargoHash = "sha256-2ipmYjUUD75VFFYBkVop0zy0fPvjHQGadLSrcCtJK4w=";

          buildInputs = with pkgs; [ dbus ];

          nativeBuildInputs = with pkgs; [
            makeWrapper
            dbus
            pkg-config
          ];

          postInstall = ''
            wrapProgram $out/bin/media-tui \
              --prefix PATH : ${pkgs.lib.makeBinPath runtimeDependencies}
          '';

          meta.mainProgram = "media-tui";
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
