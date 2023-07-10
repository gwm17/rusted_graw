use std::path::{Path, PathBuf};
use serde_derive::{Serialize, Deserialize};

use crate::error::ConfigError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub graw_path: PathBuf,
    pub hdf_path: PathBuf,
    pub pad_map_path: PathBuf,
    pub run_number: i32
}


impl Config {

    #[allow(dead_code)]
    pub fn default() -> Self {
        Self { graw_path: PathBuf::from("None"), hdf_path: PathBuf::from("None"), pad_map_path: PathBuf::from("None"), run_number: 0 }
    }

    pub fn read_config_file(config_path: &Path) -> Result<Self, ConfigError> {
        if !config_path.exists() {
            return Err(ConfigError::BadFilePath(config_path.to_path_buf()));
        }

        let yaml_str = std::fs::read_to_string(config_path)?;

        Ok(serde_yaml::from_str::<Self>(&yaml_str)?)
    }

    pub fn get_run_directory(&self) -> Result<PathBuf, ConfigError> {
        let run_dir: PathBuf = self.graw_path.join(self.get_run_str());
        if run_dir.exists() {
            return Ok(run_dir);
        } else {
            return Err(ConfigError::BadFilePath(run_dir));
        }
    }

    pub fn get_hdf_file_name(&self) -> Result<PathBuf, ConfigError> {
        let hdf_file_path: PathBuf = self.hdf_path.join(format!("{}.h5", self.get_run_str()));
        if self.hdf_path.exists() {
            return Ok(hdf_file_path);
        } else {
            return Err(ConfigError::BadFilePath(self.hdf_path.clone()));
        }
    }

    fn get_run_str(&self) -> String {
        if self.run_number < 10 {
            return format!("run_000{}", self.run_number);
         }
         else if self.run_number < 100 {
            return format!("run_00{}", self.run_number);
         }
         else if self.run_number < 1000 {
             return format!("run_0{}", self.run_number);
         }
         else {
             return format!("run_{}", self.run_number);
         }
    }
}


