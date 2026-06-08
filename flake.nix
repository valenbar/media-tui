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
        ];

        nativeBuildInputs = with pkgs; [
          makeWrapper
          dbus
          pkg-config
        ];

        buildInputs = with pkgs; [ dbus ];

        version = (fromTOML (builtins.readFile ./Cargo.toml)).package.version;
      in
      {
        devShells.default =
          with pkgs;
          mkShell {
            packages = [
              cargo
              rustc
              rustfmt
              rustPackages.clippy
              bacon
              rust-analyzer
            ]
            ++ nativeBuildInputs
            ++ buildInputs
            ++ runtimeDependencies;

            RUST_SRC_PATH = rustPlatform.rustLibSrc;
          };

        defaultPackage = pkgs.rustPlatform.buildRustPackage rec {
          inherit version nativeBuildInputs buildInputs;

          pname = "media-tui";
          src = ./.;

          cargoLock = {
            lockFile = ./Cargo.lock;
          };

          postInstall = ''
            wrapProgram $out/bin/${pname} \
              --prefix PATH : ${pkgs.lib.makeBinPath runtimeDependencies}
          '';

          meta = {
            description = "";
            homepage = "";
            license = [ ];
            mainProgram = pname;
          };
        };

      }
    );
}
