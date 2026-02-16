use std::path::Path;

use allwall::config::Config;
use anyhow::Result;
use schemars::schema_for;
use serde_json::to_string_pretty;
use tracing::info;

pub fn generate(output_dir: &Path) -> Result<()> {
    info!("Generating JSON Schema...");

    let schema = schema_for!(Config);
    let json = to_string_pretty(&schema)?;

    let schema_dir = output_dir.join("schema");
    std::fs::create_dir_all(&schema_dir)?;

    let schema_path = schema_dir.join("config.schema.json");
    std::fs::write(&schema_path, json)?;

    info!("\tWritten to: {}", schema_path.display());
    Ok(())
}
