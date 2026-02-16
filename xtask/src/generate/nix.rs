use std::path::Path;

use anyhow::Result;
use tracing::info;

pub fn generate(output_dir: &Path) -> Result<()> {
    info!("Generating NixOS module and flake...");

    let nix_dir = output_dir.join("nix");
    std::fs::create_dir_all(&nix_dir)?;

    generate_module(&nix_dir)?;
    generate_flake(&nix_dir)?;

    info!("\tModule: {}", nix_dir.join("module.nix").display());
    info!("\tFlake: {}", nix_dir.join("flake.nix").display());
    Ok(())
}

fn generate_module(nix_dir: &Path) -> Result<()> {
    let module = r#"{ config, lib, pkgs, ... }:

let
  cfg = config.services.allwall;
  configFormat = pkgs.formats.toml {};
  
  # Check if user has set any NixOS options
  hasNixOptions = cfg.fps != null 
    || cfg.gpu != null 
    || cfg.transitionType != null
    || cfg.transitionDuration != null
    || cfg.transitionInterval != null
    || cfg.scenes != [];
  
  # Generate config file from Nix options
  generatedConfig = configFormat.generate "allwall-config.toml" {
    general = lib.filterAttrs (_: v: v != null) {
      fps = cfg.fps;
      gpu = cfg.gpu;
    };
    transition = lib.filterAttrs (_: v: v != null) {
      type = cfg.transitionType;
      duration = cfg.transitionDuration;
      interval = cfg.transitionInterval;
    };
    scene = cfg.scenes;
  };
in
{
  options.services.allwall = {
    enable = lib.mkEnableOption "allwall wallpaper daemon";
    
    package = lib.mkOption {
      type = lib.types.package;
      default = pkgs.allwall;
      defaultText = lib.literalExpression "pkgs.allwall";
      description = "The allwall package to use.";
    };
    
    fps = lib.mkOption {
      type = lib.types.nullOr lib.types.int;
      default = null;
      description = ''
        Target framerate for wallpaper rendering.
        Higher values provide smoother animations but increase GPU usage.
        Recommended: 30 (balanced), 60 (smooth).
      '';
    };
    
    gpu = lib.mkOption {
      type = lib.types.nullOr lib.types.str;
      default = null;
      description = ''
        GPU selection strategy.
        - "auto": Automatically select the best GPU
        - "integrated": Force integrated graphics
        - "dedicated": Force dedicated graphics
        - "pci:VENDOR:DEVICE": Select specific GPU by PCI IDs (e.g., "pci:10de:1b80")
      '';
    };
    
    transitionType = lib.mkOption {
      type = lib.types.nullOr (lib.types.enum [
        "fade"
        "circle-top-left"
        "circle-top-right"
        "circle-bottom-left"
        "circle-bottom-right"
        "circle-center"
        "circle-random"
      ]);
      default = null;
      description = "Transition animation type.";
    };
    
    transitionDuration = lib.mkOption {
      type = lib.types.nullOr lib.types.int;
      default = null;
      description = "Duration of the transition animation in seconds.";
    };
    
    transitionInterval = lib.mkOption {
      type = lib.types.nullOr lib.types.int;
      default = null;
      description = "Time between automatic wallpaper rotations in seconds.";
    };
    
    scenes = lib.mkOption {
      type = lib.types.listOf lib.types.attrs;
      default = [];
      description = ''
        Scene configurations for monitor assignments.
        Each scene defines a wallpaper configuration for one or more monitors.
      '';
      example = lib.literalExpression ''
        [{
          path = "wallpapers/nature";
          layout = "clone";
          fit = "cover";
          monitors = "*";
        }]
      '';
    };
    
    configFile = lib.mkOption {
      type = lib.types.nullOr lib.types.path;
      default = null;
      description = ''
        Path to a custom config.toml file.
        If set, this overrides all NixOS configuration options.
      '';
    };
  };
  
  config = lib.mkIf cfg.enable {
    assertions = [
      {
        assertion = !(hasNixOptions && cfg.configFile != null);
        message = "Cannot use both NixOS options and configFile. Choose one.";
      }
    ];
    
    systemd.user.services.allwall = {
      description = "Allwall Wallpaper Daemon";
      after = [ "graphical-session.target" ];
      partOf = [ "graphical-session.target" ];
      wantedBy = [ "graphical-session.target" ];
      
      serviceConfig = {
        ExecStart = "${cfg.package}/bin/allwall run"
          + lib.optionalString (cfg.configFile != null) " --config ${cfg.configFile}"
          + lib.optionalString (hasNixOptions && cfg.configFile == null) " --config ${generatedConfig}";
        Restart = "on-failure";
        RestartSec = "5";
      };
    };
  };
}
"#;

    std::fs::write(nix_dir.join("module.nix"), module)?;
    Ok(())
}

fn generate_flake(nix_dir: &Path) -> Result<()> {
    let flake = r#"{ 
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
        pkgs = import nixpkgs {
          inherit system overlays;
        };
        
        rustToolchain = pkgs.rust-bin.stable.latest.default;
        
        buildInputs = with pkgs; [
          wayland
          libxkbcommon
          libGL
          gst_all_1.gstreamer
          gst_all_1.gst-plugins-base
          gst_all_1.gst-plugins-good
        ];
        
        nativeBuildInputs = with pkgs; [
          rustToolchain
          pkg-config
          makeWrapper
        ];
        
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
      in
      {
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
          
          module-syntax = pkgs.runCommand "check-module-syntax" {} ''
            ${pkgs.nix}/bin/nix-instantiate --parse ${./module.nix} > /dev/null
            touch $out
          '';
        };
      }
    );
}
"#;

    std::fs::write(nix_dir.join("flake.nix"), flake)?;
    Ok(())
}
