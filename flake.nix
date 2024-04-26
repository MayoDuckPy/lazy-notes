{
  description = "A basic Rust devshell for NixOS users developing Leptos";

  inputs = {
    nixpkgs.url      = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url  = "github:numtide/flake-utils";

    rust-overlay.url = "github:oxalica/rust-overlay";
    crane = {
      url = "github:ipetkov/crane";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay, crane, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        name = "lazy-notes";
        listen-port = "3000";
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
          config.allowUnfree = true;
        };

        settings-filter = path: type: builtins.match "settings\.toml" path == null;
        css-filter = path: type:
          builtins.match "styles/.*\.css$" path != null && type == "file";
        src-filter = path: type:
          (settings-filter path type)
          || (css-filter path type)
          || (craneLib.filterCargoSources path type);

        rust-toolchain = pkgs.rust-bin.selectLatestNightlyWith(
          toolchain: toolchain.default.override {
            extensions = [ "rust-src" "rust-analyzer" ];
            targets = [
              "aarch64-linux-android"
              "armv7-linux-androideabi"
              "i686-linux-android"
              "x86_64-linux-android"
              "wasm32-unknown-unknown"
            ];
          }
        );
        craneLib = crane.lib.${system}.overrideToolchain rust-toolchain;

        commonArgs = {
          src = pkgs.lib.cleanSourceWith {
            src = craneLib.path ./server;
            filter = src-filter;
          };

          strictDeps = true;
          buildInputs = with pkgs; [
            openssl
          ];
        };

        cargoArtifacts = craneLib.buildDepsOnly (commonArgs // {
          pname = name;

          nativeBuildInputs = with pkgs; [
            openssl
            pkg-config
          ];
        });

        lazy-notes-server = craneLib.buildPackage (commonArgs // {
          inherit cargoArtifacts;

          nativeBuildInputs = with pkgs; [
            makeWrapper
            openssl
            pkg-config

            binaryen
            cargo-leptos
          ];

          cargoExtraArgs = "";
          cargoTestExtraArgs = "";
          cargoTestCommand = "cargo leptos test -r";
          buildPhaseCargoCommand = "cargo leptos build -r";

          # Copy release binary and site root
          installPhaseCommand = ''
            mkdir -p $out/bin
            mkdir -p $out/etc

            cp target/release/${name} $out/bin
            cp -r target/site $out/etc/${name}

            wrapProgram $out/bin/${name} \
              --set LEPTOS_SITE_ROOT $out/etc/${name} \
              --set LEPTOS_SITE_ADDR 0.0.0.0:${listen-port}
          '';
        });
      in
      {
        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            # Rust & Leptos
            openssl
            pkg-config
            cacert
            cargo-leptos
            cargo-make
            trunk
            rust-toolchain

            # Mobile Dev
            androidStudioPackages.dev
            corepack
            kotlin-language-server
            ktlint
            python3
          ];

          shellHook = "";
        };

        packages = {
          server = lazy-notes-server;

          dockerImage = pkgs.dockerTools.buildLayeredImage {
            inherit name;
            tag = "latest";
            contents = [ lazy-notes-server pkgs.coreutils ];

            config = {
              User = "1000:1000";
              Cmd = [ name ];
              ExposedPorts = {
                "${listen-port}/tcp" = { };
              };
              Volumes = { "/data" = { }; };
            };
          };
        };
      }
    );
}
