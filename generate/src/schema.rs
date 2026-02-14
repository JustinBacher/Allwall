use std::path::Path;

use allwall::config::Config;
use log::info;
use schemars::schema_for;
use serde_json::to_string_pretty;

pub fn generate(output_dir: &Path) {
	info!("Generating JSON Schema...");

	let schema = schema_for!(Config);
	let json = to_string_pretty(&schema).expect("Failed to serialize schema");

	let schema_dir = output_dir.join("schema");
	std::fs::create_dir_all(&schema_dir).expect("Failed to create schema directory");

	let schema_path = schema_dir.join("config.schema.json");
	std::fs::write(&schema_path, json).expect("Failed to write schema");

	info!("\tWritten to: {}", schema_path.display());
}
