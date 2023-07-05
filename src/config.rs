use std::path::{Path, PathBuf};
use serde_derive::{Serialize, Deserialize};

use crate::error::ConfigError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub graw_path: PathBuf,
    pub hdf_path: PathBuf,
    pub pad_map_path: PathBuf
}


impl Config {

    #[allow(dead_code)]
    pub fn default() -> Self {
        Self { graw_path: PathBuf::from("None"), hdf_path: PathBuf::from("None"), pad_map_path: PathBuf::from("None") }
    }

    pub fn read_config_file(config_path: &Path) -> Result<Self, ConfigError> {
        if !config_path.exists() {
            return Err(ConfigError::BadFilePath(config_path.to_path_buf()));
        }

        let yaml_str = std::fs::read_to_string(config_path)?;

        Ok(serde_yaml::from_str::<Self>(&yaml_str)?)
    }
}


