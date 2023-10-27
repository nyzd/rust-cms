use serde::{Deserialize, Serialize};
use serde_json;

use crate::manager::ManagerError;

#[derive(Default, Debug, Clone, Deserialize, Serialize)]
pub enum PluginAbiParamType {
    #[serde(rename = "string")]
    String,

    #[serde(rename = "number")]
    #[default]
    Number,
}

#[derive(Default, Debug, Clone, Deserialize, Serialize)]
pub struct PluginAbiResult {
    #[serde(rename = "type")]
    pub ty: PluginAbiParamType,
}

#[derive(Default, Debug, Clone, Deserialize, Serialize)]
pub struct PluginAbiFunction {
    pub name: String,
    //params: Vec<>
    pub result: PluginAbiResult,
}

#[derive(Default, Debug, Clone, Deserialize, Serialize)]
pub struct PluginAbi {
    pub functions: Vec<PluginAbiFunction>,
}

#[derive(Default, Debug, Clone, Deserialize, Serialize)]
pub struct PluginConfig<T> {
    #[serde(flatten)]
    pub metadata: T,
    pub abi: PluginAbi,
}

impl<T> TryFrom<Vec<u8>> for PluginConfig<T>
where
    T: for<'a> Deserialize<'a>,
{
    type Error = ManagerError;
    fn try_from(bytes: Vec<u8>) -> Result<Self, Self::Error> {
        let Ok(config) = serde_json::from_slice::<PluginConfig<T>>(&bytes) else {
            return Err(ManagerError::Config("Cant parse the config file as json!".to_string()))
        };

        Ok(config)
    }
}
