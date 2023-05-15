use serde::{Deserialize, Serialize};
use serde_json;
use std::{fs::File, io::Read, path::PathBuf};

use crate::manager::ManagerError;

#[derive(Deserialize, Serialize)]
pub struct PluginConfig<T> {
    #[serde(flatten)]
    pub metadata: T,
    pub wasm_path: PathBuf,
}

impl<T> PluginConfig<T>
where
    T: for<'a> Deserialize<'a>,
{
    pub fn from_file(file_path: PathBuf) -> Result<Self, ManagerError> {
        // TODO: This may have blocking issue

        // Read the file
        let Ok(mut file) = File::open(file_path) else {
            return Err(ManagerError::Config("Cant open the config file!".to_string()))
        };

        let mut file_content = String::new();
        let Ok(_reader) = file.read_to_string(&mut file_content) else {
            return Err(ManagerError::Config("Cant read the config file!".to_string()))
        };

        let Ok(config) = serde_json::from_str::<PluginConfig<T>>(&file_content) else {
            return Err(ManagerError::Config("Cant parse the config file as json!".to_string()))
        };

        Ok(config)
    }
}
