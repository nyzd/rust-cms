use std::{
    collections::HashMap,
    fs::{canonicalize, File},
    io::Read,
    path::PathBuf,
    vec,
};

use serde::Deserialize;

use crate::{config::PluginConfig, wasm::WasmPlugin};

/// The plugin metadata that will
/// mostly shown to the user
#[derive(Clone, Debug, Deserialize)]
pub struct PluginMetadata {
    pub name: String,
    pub version: String,
}

#[derive(Clone, Debug)]
pub enum PluginError {
    Build(String),
}

/// The trait for building a plugin
pub trait Plugin<T> {
    /// Metadata of the plugin
    fn metadata(&self) -> PluginMetadata;

    /// The wasm source of the plugin
    fn source(&self) -> String;

    fn build(&self) -> Result<T, PluginError>;

    /// Permissions plugin requires
    fn permissions(&self) -> Vec<String>;

    /// Routers specific to the plugin
    ///
    /// for example -> get_data is a router
    /// that point outs to the function called `get_data`
    ///
    /// The PluginManager will run the this functions if
    /// incoming request is pointing out to that path
    ///
    /// for example there is a incoming request that sended to the
    /// `/extentions/extention_name/get_data`
    ///
    /// the manager will run the `get_data` function and anything that is returned
    /// form that function will sent as a response
    ///
    /// we can get data from the function return in the source of plugin
    /// or anything in the config file
    ///
    /// for example:
    ///
    /// ```javascript
    /// function routers() {
    ///     return ["get_data"];
    /// }
    ///
    /// function get_data(req) {
    ///     return "response"
    /// }
    /// ```
    ///
    /// or we can get this data from the list of functions
    fn routers(&self) -> Vec<String>;
}

#[derive(Clone, Debug)]
pub struct PluginBuilder {
    metadata: PluginMetadata,
    source: String,
}

impl PluginBuilder {
    /// Creates a new PluginBuilder
    pub fn new(metadata: PluginMetadata, source: String) -> Self {
        Self { metadata, source }
    }
}

impl Plugin<WasmPlugin> for PluginBuilder {
    fn metadata(&self) -> PluginMetadata {
        self.metadata.clone()
    }

    fn source(&self) -> String {
        self.source.clone()
    }

    fn permissions(&self) -> Vec<String> {
        vec![]
    }

    fn routers(&self) -> Vec<String> {
        vec![]
    }

    fn build(&self) -> Result<WasmPlugin, PluginError> {
        // Create a new WasmPlugin
        let Ok(wasm_plugin) = WasmPlugin::new(self.source()) else {
            return Err(
                PluginError::Build(
                    format!("Cant build plugin {} from the source", self.metadata().name)
                )
            );
        };

        Ok(wasm_plugin)
    }
}

/// This is a error type that the plugin manager
/// may return
#[derive(Clone, Debug)]
pub enum ManagerError {
    Source(String),
    Config(String),
}

/// This is a manager for our plugins
/// for example for gathering the plugins data
/// or plugin source
/// or updating the plugin
/// install and delete the plugin
#[derive(Clone, Debug)]
pub struct PluginManager<T> {
    plugins: HashMap<String, T>,
}

impl<T> PluginManager<T> {
    /// Creates and returns the new PluginManager
    pub fn new() -> Self {
        Self {
            plugins: HashMap::new(),
        }
    }
}

impl PluginManager<PluginBuilder> {
    pub fn add_from_config(
        &mut self,
        config_path: PathBuf,
        config: PluginConfig<PluginMetadata>,
    ) -> Result<&mut Self, ManagerError> {
        // Get source of the plugin from path that is defined in the config
        // TODO: this may has a blocking issue
        let Ok(mut file)= File::open(
            canonicalize(
                PathBuf::from(format!("{}{}", config_path.to_string_lossy(), config.wasm_path.to_string_lossy())
            )
        ).unwrap()) else {
            return Err(ManagerError::Source("Cant open the source file!".to_string()))
        };

        let mut file_content = String::new();
        let Ok(_reader) = file.read_to_string(&mut file_content) else {
            return Err(ManagerError::Source("Cant read the source file!".to_string()))
        };

        // Build the plugin with pluginBuilder
        let new_plugin = PluginBuilder::new(config.metadata, file_content);

        self.plugins.insert(new_plugin.metadata().name, new_plugin);

        Ok(self)
    }
}

impl<T> PluginManager<T>
where
    T: Plugin<WasmPlugin>,
{
    /// Adds the new plugin to the plugins list(map)
    pub fn add(&mut self, plugin: T) -> &mut Self {
        // TODO: handle the option
        self.plugins.insert(plugin.metadata().name, plugin);

        self
    }

    /// Removes the plugin from the plugins list(map)
    pub fn remove(&mut self, plugin: T) -> &mut Self {
        // TODO: handle the option
        self.plugins.remove(&plugin.metadata().name);

        self
    }

    /// Returns the list of the plugins on mem
    pub fn get_the_plugins_list(&self) -> Vec<PluginMetadata> {
        self.plugins
            .values()
            .into_iter()
            .map(|m| m.metadata())
            .collect::<Vec<PluginMetadata>>()
    }

    /// Finds the plugin from given name
    pub fn get_plugin(&self, name: String) -> Option<&T> {
        self.plugins.get(&name)
    }
}
