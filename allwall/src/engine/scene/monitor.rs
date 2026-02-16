use std::borrow::Cow;

#[cfg(feature = "generate")]
use schemars::{JsonSchema, json_schema};
use serde::{Deserialize, Deserializer};
use wayland_client::protocol::wl_output::WlOutput;

/// A handle to a specific monitor
///
/// Contains the monitor's Wayland name (e.g., "DP-1", "HDMI-A-1")
/// and optionally the actual Wayland output object.
#[derive(Clone, Debug, PartialEq)]
pub struct MonitorHandle {
    /// Monitor name as reported by Wayland (e.g., "DP-1", "HDMI-A-1")
    name: String,

    /// Wayland output handle (populated during engine initialization)
    output: Option<WlOutput>,
}

impl MonitorHandle {
    pub fn new(name: String) -> Self {
        Self { name, output: None }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn output(&self) -> Option<&WlOutput> {
        self.output.as_ref()
    }

    pub fn set_output(&mut self, output: WlOutput) {
        self.output = Some(output);
    }
}

impl std::hash::Hash for MonitorHandle {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}

impl Eq for MonitorHandle {}

/// Monitor specification for scene assignment
///
/// Specifies which monitors a scene should render to.
/// Supports flexible specification formats for convenience.
///
/// # Examples
///
/// ```toml
/// # All monitors
/// monitors = "*"
/// monitors = "any"
///
/// # Single monitor by name
/// monitors = "DP-1"
///
/// # Multiple specific monitors
/// monitors = ["DP-1", "HDMI-A-1"]
///
/// # Array with wildcard (resolves to all)
/// monitors = ["DP-1", "*"]
/// ```
#[derive(Clone, Debug, PartialEq, Default)]
pub enum MonitorsSpec {
    /// Apply to all available monitors
    #[default]
    Any,

    /// Apply to specific monitors by name
    Specific(Vec<MonitorHandle>),
}

impl MonitorsSpec {
    pub fn is_any(&self) -> bool {
        matches!(self, MonitorsSpec::Any)
    }

    pub fn monitors(&self) -> Option<&[MonitorHandle]> {
        match self {
            MonitorsSpec::Any => None,
            MonitorsSpec::Specific(handles) => Some(handles),
        }
    }
}

impl<'de> Deserialize<'de> for MonitorsSpec {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::de::{self, Visitor};

        struct MonitorsSpecVisitor;

        impl<'de> Visitor<'de> for MonitorsSpecVisitor {
            type Value = MonitorsSpec;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a string (\"*\", \"any\", or monitor name) or an array of strings")
            }

            fn visit_str<E>(self, v: &str) -> Result<MonitorsSpec, E>
            where
                E: de::Error,
            {
                let lower = v.to_lowercase();
                if lower == "*" || lower == "any" {
                    Ok(MonitorsSpec::Any)
                } else {
                    Ok(MonitorsSpec::Specific(vec![MonitorHandle::new(v.to_string())]))
                }
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<MonitorsSpec, A::Error>
            where
                A: serde::de::SeqAccess<'de>,
            {
                let mut handles = Vec::new();

                while let Some(item) = seq.next_element::<String>()? {
                    let lower = item.to_lowercase();
                    if lower == "*" || lower == "any" {
                        return Ok(MonitorsSpec::Any);
                    }
                    handles.push(MonitorHandle::new(item));
                }

                if handles.is_empty() {
                    Ok(MonitorsSpec::Any)
                } else {
                    Ok(MonitorsSpec::Specific(handles))
                }
            }
        }

        deserializer.deserialize_any(MonitorsSpecVisitor)
    }
}

#[cfg(feature = "generate")]
impl JsonSchema for MonitorsSpec {
    fn schema_name() -> Cow<'static, str> {
        Cow::Borrowed("MonitorsSpec")
    }

    fn schema_id() -> Cow<'static, str> {
        Cow::Borrowed(concat!(module_path!(), "::MonitorsSpec"))
    }

    fn json_schema(_: &mut schemars::SchemaGenerator) -> schemars::Schema {
        json_schema!({
            "description": "Monitor specification. Use '*' or 'any' for all monitors, a single name like 'DP-1', or an array like ['DP-1', 'HDMI-A-1'].",
            "oneOf": [
                {
                    "type": "string",
                    "description": "Single monitor name, or '*'/'any' for all monitors"
                },
                {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "Array of monitor names. If any element is '*' or 'any', resolves to all monitors."
                }
            ]
        })
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[derive(serde::Deserialize)]
    struct TestConfig {
        monitors: MonitorsSpec,
    }

    #[test]
    fn test_deserialize_star() {
        let config: TestConfig = toml::from_str(r#"monitors = "*""#).unwrap();
        assert_eq!(config.monitors, MonitorsSpec::Any);
    }

    #[test]
    fn test_deserialize_any() {
        let config: TestConfig = toml::from_str(r#"monitors = "any""#).unwrap();
        assert_eq!(config.monitors, MonitorsSpec::Any);
    }

    #[test]
    fn test_deserialize_single_monitor() {
        let config: TestConfig = toml::from_str(r#"monitors = "DP-1""#).unwrap();
        assert_eq!(
            config.monitors,
            MonitorsSpec::Specific(vec![MonitorHandle::new("DP-1".to_string())])
        );
    }

    #[test]
    fn test_deserialize_array_with_star() {
        let config: TestConfig = toml::from_str(r#"monitors = ["DP-1", "*"]"#).unwrap();
        assert_eq!(config.monitors, MonitorsSpec::Any);
    }
}
