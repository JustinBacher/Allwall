use std::{
    borrow::Cow::{self, Borrowed},
    result::Result,
};

#[cfg(feature = "generate")]
use schemars::{JsonSchema, json_schema};
use serde::{Deserialize, Deserializer};
#[cfg(feature = "generate")]
use serde_nixos::{self, NixosType};

/// GPU selection strategy for rendering
///
/// Determines which GPU to use for wallpaper rendering.
/// This is particularly useful on systems with both integrated and dedicated GPUs.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "generate", derive(serde_nixos::NixosType))]
#[derive(Default)]
pub enum GpuSelection {
    /// Automatically select the best available GPU
    ///
    /// Prefers dedicated GPU over integrated when available.
    #[default]
    Auto,

    /// Force use of integrated graphics
    ///
    /// Useful for power saving on laptops with hybrid graphics.
    Integrated,

    /// Force use of dedicated graphics
    ///
    /// Required for some multi-monitor setups or when integrated
    /// graphics don't support required features.
    Dedicated,

    /// Select a specific GPU by PCI IDs
    ///
    /// Use `lspci -nn` to find the vendor and device IDs.
    /// Format: `pci:VENDOR:DEVICE` (hexadecimal, e.g., `pci:10de:1b80`)
    Pci { vendor: u16, device: u16 },
}

impl<'de> Deserialize<'de> for GpuSelection {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        match String::deserialize(deserializer)?.to_lowercase().as_str() {
            "auto" => Ok(GpuSelection::Auto),
            "integrated" => Ok(GpuSelection::Integrated),
            "dedicated" => Ok(GpuSelection::Dedicated),
            s => {
                if let Some(pci_str) = s.strip_prefix("pci:") {
                    let parts: Vec<&str> = pci_str.split(':').collect();
                    if parts.len() == 2 {
                        let vendor = u16::from_str_radix(parts[0], 16)
                            .map_err(|_| serde::de::Error::custom("Invalid PCI vendor ID"))?;
                        let device = u16::from_str_radix(parts[1], 16)
                            .map_err(|_| serde::de::Error::custom("Invalid PCI device ID"))?;
                        return Ok(GpuSelection::Pci { vendor, device });
                    }
                }
                Err(serde::de::Error::custom(format!(
                    "Invalid gpu value: {}. Expected 'auto', 'integrated', 'dedicated', or 'pci:vendor:device'",
                    s
                )))
            },
        }
    }
}

#[cfg(feature = "generate")]
impl JsonSchema for GpuSelection {
    fn schema_name() -> Cow<'static, str> {
        Borrowed("GpuSelection")
    }

    fn schema_id() -> Cow<'static, str> {
        Borrowed(concat!(module_path!(), "::GpuSelection"))
    }

    fn json_schema(_: &mut schemars::SchemaGenerator) -> schemars::Schema {
        json_schema!({
            "type": "string",
            "pattern": "auto|integrated|dedicated|pci:[0-9a-fA-F]{4}:[0-9a-fA-F]{4}",
            "description": "GPU selection strategy. Use 'auto', 'integrated', 'dedicated', or 'pci:VENDOR:DEVICE' (hex IDs, e.g., 'pci:10de:1b80')"
        })
    }
}

/// General engine configuration options
#[derive(Debug, Clone, Deserialize)]
#[cfg_attr(feature = "generate", derive(JsonSchema, NixosType))]
pub struct GeneralConfig {
    /// Target framerate for wallpaper rendering
    ///
    /// Higher values provide smoother animations but increase GPU usage.
    /// Recommended values: 30 (balanced), 60 (smooth), 144+ (high refresh rate monitors).
    #[serde(default = "default_fps")]
    #[cfg_attr(feature = "generate", schemars(default = "default_fps"))]
    #[cfg_attr(feature = "generate", nixos(default = "30"))]
    pub fps: u32,

    /// GPU selection strategy
    #[serde(default)]
    pub gpu: GpuSelection,
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            fps: default_fps(),
            gpu: GpuSelection::Auto,
        }
    }
}

fn default_fps() -> u32 {
    30
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[derive(serde::Deserialize)]
    struct GpuConfig {
        gpu: GpuSelection,
    }

    #[test]
    fn test_gpu_selection_auto() {
        let config: GpuConfig = toml::from_str(r#"gpu = "auto""#).unwrap();
        assert_eq!(config.gpu, GpuSelection::Auto);
    }

    #[test]
    fn test_gpu_selection_integrated() {
        let config: GpuConfig = toml::from_str(r#"gpu = "integrated""#).unwrap();
        assert_eq!(config.gpu, GpuSelection::Integrated);
    }

    #[test]
    fn test_gpu_selection_dedicated() {
        let config: GpuConfig = toml::from_str(r#"gpu = "dedicated""#).unwrap();
        assert_eq!(config.gpu, GpuSelection::Dedicated);
    }

    #[test]
    fn test_gpu_selection_pci_uppercase() {
        let config: GpuConfig = toml::from_str(r#"gpu = "pci:10de:1B80""#).unwrap();
        assert_eq!(
            config.gpu,
            GpuSelection::Pci {
                vendor: 0x10de,
                device: 0x1B80
            }
        );
    }

    #[test]
    fn test_gpu_selection_pci_lowercase() {
        let config: GpuConfig = toml::from_str(r#"gpu = "pci:10de:1b80""#).unwrap();
        assert_eq!(
            config.gpu,
            GpuSelection::Pci {
                vendor: 0x10de,
                device: 0x1b80
            }
        );
    }

    #[test]
    fn test_gpu_selection_invalid() {
        let result = toml::from_str::<GpuConfig>(r#"gpu = "invalid""#);
        assert!(result.is_err());
    }

    #[test]
    fn test_gpu_selection_pci_invalid_format() {
        let result = toml::from_str::<GpuConfig>(r#"gpu = "pci:invalid""#);
        assert!(result.is_err());
    }

    #[test]
    fn test_general_config_defaults() {
        let config = GeneralConfig::default();
        assert_eq!(config.fps, 30);
        assert_eq!(config.gpu, GpuSelection::Auto);
    }

    #[test]
    fn test_general_config_deserialize() {
        let config: GeneralConfig = toml::from_str(r#"fps = 60"#).unwrap();
        assert_eq!(config.fps, 60);
        assert_eq!(config.gpu, GpuSelection::Auto);
    }
}
