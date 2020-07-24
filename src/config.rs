use serde::{Deserialize, Serialize};

use std::path::PathBuf;
use std::collections::HashSet;

use super::error::DumperError;

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub input_pbf: PathBuf,
    pub output_db: PathBuf,
    #[serde(default)]
    pub overwrite_output: bool,
    #[serde(default)]
    pub skip_tag_keys: HashSet<String>,
    #[serde(default)]
    pub skip_nodes: bool,
    #[serde(default)]
    pub skip_node_info: bool,
    #[serde(default)]
    pub skip_node_tags: bool,
    #[serde(default)]
    pub skip_relations: bool,
    #[serde(default)]
    pub skip_relation_info: bool,
    #[serde(default)]
    pub skip_relation_members: bool,
    #[serde(default)]
    pub skip_relation_tags: bool,
    #[serde(default)]
    pub skip_ways: bool,
    #[serde(default)]
    pub skip_way_info: bool,
    #[serde(default)]
    pub skip_way_refs: bool,
    #[serde(default)]
    pub skip_way_tags: bool,
}

pub fn read_config(config_path: String) -> Result<Config, DumperError> {
    let config_contents = std::fs::read_to_string(&config_path)
        .map_err(|err| DumperError::new(err.into(), format!("Failed to read configuration from `{}`", config_path)))?;

    let config = serde_json::from_str::<Config>(&config_contents)
        .map_err(|err| DumperError::new(err.into(), format!("Failed to parse configuration from `{}`", config_path)))?;

    Ok(config)
}
