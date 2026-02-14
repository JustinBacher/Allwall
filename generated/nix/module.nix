{ config, lib, pkgs, ... }:

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
