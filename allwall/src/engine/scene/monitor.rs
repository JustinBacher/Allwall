use std::borrow::Cow;

#[cfg(feature = "generate")]
use schemars::{JsonSchema, json_schema};
use serde::{Deserialize, Deserializer};
use smithay_client_toolkit::{
    output::OutputInfo, reexports::client::protocol::wl_output::WlOutput, shell::wlr_layer::LayerSurface,
};

/// A handle to a specific monitor
#[derive(Clone, Debug, PartialEq)]
pub struct MonitorHandle {
    name: String,
}

impl MonitorHandle {
    pub fn new(name: String) -> Self {
        Self { name }
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

impl std::hash::Hash for MonitorHandle {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}

impl Eq for MonitorHandle {}

impl std::fmt::Display for MonitorHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

/// Runtime monitor with layer surface
pub struct Monitor {
    handle: MonitorHandle,
    layer: LayerSurface,
    output: Option<WlOutput>,
    info: Option<OutputInfo>,
}

impl Monitor {
    pub fn new(handle: MonitorHandle, layer: LayerSurface, output: WlOutput, info: OutputInfo) -> Self {
        Self {
            handle,
            layer,
            output: Some(output),
            info: Some(info),
        }
    }

    pub fn simple(handle: MonitorHandle, layer: LayerSurface) -> Self {
        Self {
            handle,
            layer,
            output: None,
            info: None,
        }
    }

    pub fn handle(&self) -> &MonitorHandle {
        &self.handle
    }

    pub fn layer(&self) -> &LayerSurface {
        &self.layer
    }

    pub fn output(&self) -> Option<&WlOutput> {
        self.output.as_ref()
    }

    pub fn info(&self) -> Option<&OutputInfo> {
        self.info.as_ref()
    }

    pub fn size(&self) -> (u32, u32) {
        self.info
            .as_ref()
            .and_then(|i| i.logical_size)
            .map(|(w, h)| (w as u32, h as u32))
            .unwrap_or((1920, 1080))
    }

    pub fn scale_factor(&self) -> i32 {
        self.info.as_ref().map(|i| i.scale_factor).unwrap_or(1)
    }
}

impl std::fmt::Debug for Monitor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Monitor")
            .field("handle", &self.handle)
            .field("size", &self.size())
            .field("scale", &self.scale_factor())
            .finish()
    }
}

/// Monitor specification for scene assignment
#[derive(Clone, Debug, PartialEq, Default)]
pub enum MonitorsSpec {
    #[default]
    Any,
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

    pub fn matches(&self, name: &str) -> bool {
        match self {
            MonitorsSpec::Any => true,
            MonitorsSpec::Specific(handles) => handles.iter().any(|h| h.name() == name),
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
