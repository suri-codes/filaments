{
  description = "Filaments dev flake";

  # Flake inputs
  inputs = {

    nixpkgs.url = "https://flakehub.com/f/NixOS/nixpkgs/0.1"; # unstable Nixpkgs

    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  # Flake outputs
  outputs =
    { self, ... }@inputs:

    let
      # The systems supported for this flake
      supportedSystems = [
        "x86_64-linux" # 64-bit Intel/AMD Linux
        "aarch64-linux" # 64-bit ARM Linux
        "x86_64-darwin" # 64-bit Intel macOS
        "aarch64-darwin" # 64-bit ARM macOS
      ];

      # Helper to provide system-specific attributes
      forEachSupportedSystem =
        f:
        inputs.nixpkgs.lib.genAttrs supportedSystems (
          system:
          f {
            pkgs = import inputs.nixpkgs {
              inherit system;
              overlays = [
                inputs.self.overlays.default
              ];
            };
          }
        );
    in
    {

      overlays.default = final: prev: {

        sea-orm-cli = final.rustPlatform.buildRustPackage rec {
          pname = "sea-orm-cli";
          version = "2.0.0-rc.37";

          src = final.fetchCrate {
            inherit pname version;

            sha256 = "sha256-YbP85rVO41S7ZPWSpVz3jICLAEU8H/a2axJBtdFRuWY=";

          };

          cargoHash = "sha256-6lOXyaNxrIfCI3T9nIPR76rhQXvRzSVQUsPRjo5abmI=";

          nativeBuildInputs = [ final.pkg-config ];

          buildInputs = [
            final.openssl
          ];

          doCheck = false; # Skip tests to speed up the build
        };

        rustToolchain =
          with inputs.fenix.packages.${prev.stdenv.hostPlatform.system};
          combine (
            with stable;
            [
              clippy
              rustc
              cargo
              rustfmt
              rust-src
            ]
          );
      };

      packages = forEachSupportedSystem (
        { pkgs }:
        {
          default = pkgs.rustPlatform.buildRustPackage {
            pname = "fil";
            version = "0.1.0";
            src = ./.;
            cargoLock.lockFile = ./Cargo.lock;
            cargoLock.outputHashes = {
              "egui_graphs-0.30.0" = "sha256-mJuGENPnDeD8SDNBXZZpLwcqrKlLTxuM6yCCx+1dSko=";
            };

            nativeBuildInputs = [ pkgs.pkg-config ];
            buildInputs = [ pkgs.openssl ];
          };
        }
      );

      devShells = forEachSupportedSystem (
        { pkgs }:
        {
          default = pkgs.mkShellNoCC {
            # The Nix packages provided in the environment
            # Add any you need here
            packages = with pkgs; [

              rustToolchain
              openssl
              pkg-config
              cargo-deny
              cargo-edit
              cargo-watch
              cargo-nextest

              rust-analyzer

              sea-orm-cli

              bacon
            ];

            # Set any environment variables for your dev shell
            env = {
              RUST_SRC_PATH = "${pkgs.rustToolchain}/lib/rustlib/src/rust/library";

              # project specific vars
              FIL_CONFIG = "./.config";
              FIL_DATA = "./.data";
              FIL_LOG_LEVEL = "DEBUG";
            };

            # Add any shell logic you want executed any time the environment is activated
            shellHook = "";
          };
        }
      );
    };
}
