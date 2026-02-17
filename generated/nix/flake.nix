{
  description = "Allwall - High-performance Wayland wallpaper renderer";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };

        rustToolchain = pkgs.rust-bin.stable.latest.default;

        buildInputs = with pkgs; [
          wayland
          libxkbcommon
          libGL
          gst_all_1.gstreamer
          gst_all_1.gst-plugins-base
          gst_all_1.gst-plugins-good
        ];

        nativeBuildInputs = with pkgs; [ rustToolchain pkg-config makeWrapper ];

        allwall = pkgs.rustPlatform.buildRustPackage {
          pname = "allwall";
          version = "0.1.0";
          src = ./..;
          cargoLock.lockFile = ./../Cargo.lock;

          inherit buildInputs nativeBuildInputs;

          postFixup = ''
            wrapProgram $out/bin/allwall \
              --prefix GST_PLUGIN_SYSTEM_PATH_1_0 : "${pkgs.gst_all_1.gstreamer.out}/lib/gstreamer-1.0"
          '';
        };
      in {
        packages = {
          default = allwall;
          allwall = allwall;
        };

        overlays.default = final: prev: {
          allwall = final.callPackage ./package.nix { };
        };

        nixosModules.default = import ./module.nix;

        devShells.default = pkgs.mkShell {
          packages = with pkgs; [
            rustToolchain
            rust-analyzer
            pkg-config
            wayland
            libxkbcommon
            gst_all_1.gstreamer
            gst_all_1.gst-plugins-base
          ];

          LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath buildInputs;
          RUST_SRC_PATH = "${rustToolchain}/lib/rustlib/src/rust/library";
        };

        checks = {
          build = allwall;

          module-syntax = pkgs.runCommand "check-module-syntax" { } ''
            ${pkgs.nix}/bin/nix-instantiate --parse ${./module.nix} > /dev/null
            touch $out
          '';
        };
      });
}
