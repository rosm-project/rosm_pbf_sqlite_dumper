use serde::{Deserialize, Serialize};

use std::path::PathBuf;
use std::collections::HashSet;

use super::error::DumperError;

#[derive(Default, Serialize, Deserialize)]
pub struct TableConfig {
    #[serde(default)]
    pub skip: bool,
}

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub input_pbf: PathBuf,

    pub output_db: PathBuf,

    #[serde(default)]
    pub overwrite_output: bool,

    #[serde(default)]
    pub skip_tag_keys: HashSet<String>,

    #[serde(default)]
    pub header: TableConfig,

    #[serde(default)]
    pub nodes: TableConfig,

    #[serde(default)]
    pub node_info: TableConfig,

    #[serde(default)]
    pub node_tags: TableConfig,

    #[serde(default)]
    pub relations: TableConfig,

    #[serde(default)]
    pub relation_info: TableConfig,

    #[serde(default)]
    pub relation_members: TableConfig,

    #[serde(default)]
    pub relation_tags: TableConfig,

    #[serde(default)]
    pub ways: TableConfig,

    #[serde(default)]
    pub way_info: TableConfig,

    #[serde(default)]
    pub way_refs: TableConfig,

    #[serde(default)]
    pub way_tags: TableConfig,

}

pub fn read_config(config_path: String) -> Result<Config, DumperError> {
    let config_contents = std::fs::read_to_string(&config_path)
        .map_err(|err| DumperError::new(err.into(), format!("Failed to read configuration from `{}`", config_path)))?;

    let config = serde_json::from_str::<Config>(&config_contents)
        .map_err(|err| DumperError::new(err.into(), format!("Failed to parse configuration from `{}`", config_path)))?;

    Ok(config)
}
