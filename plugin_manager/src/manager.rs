use std::collections::HashMap;

use left_right::{Absorb, ReadHandle, ReadHandleFactory, WriteHandle};
use serde::Deserialize;
use wasmer::Imports;

use crate::{
    config::{PluginAbi, PluginConfig},
    wasm::WasmPlugin,
};

/// The plugin metadata that will
/// mostly shown to the user
#[derive(Default, Clone, Debug, Deserialize)]
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

    fn abi(&self) -> PluginAbi;

    /// The wasm source of the plugin
    fn source(&self) -> Vec<u8>;

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

#[derive(Default, Clone, Debug)]
pub struct PluginBuilder {
    config: PluginConfig<PluginMetadata>,
    source: Vec<u8>,
}

impl PluginBuilder {
    /// Creates a new PluginBuilder
    pub fn new(config: PluginConfig<PluginMetadata>, source: Vec<u8>) -> Self {
        Self { config, source }
    }
}

impl Plugin<WasmPlugin> for PluginBuilder {
    fn metadata(&self) -> PluginMetadata {
        self.config.metadata.clone()
    }

    fn abi(&self) -> PluginAbi {
        self.config.abi.clone()
    }

    fn source(&self) -> Vec<u8> {
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
        let wasm_plugin = WasmPlugin::new(self.source());

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

/// ManagerOperations
/// log for left-write
#[derive(Clone, Debug)]
pub enum SystemOp<T>
where
    T: Plugin<WasmPlugin>,
{
    New(String, T),
    Remove(String),
}

/// This is a manager for our plugins
/// for example for gathering the plugins data
/// or plugin source
/// or updating the plugin
/// install and delete the plugin
#[derive(Default, Debug, Clone)]
pub struct PluginSystem<T> {
    plugins: HashMap<String, T>,
}

impl<T> PluginSystem<T>
where
    T: Plugin<WasmPlugin> + Clone + Default,
{
    pub fn get_left_right() -> (
        WriteHandle<PluginSystem<T>, SystemOp<T>>,
        ReadHandle<PluginSystem<T>>,
    ) {
        left_right::new::<PluginSystem<T>, SystemOp<T>>()
    }
}

impl<T> Absorb<SystemOp<T>> for PluginSystem<T>
where
    T: Plugin<WasmPlugin> + Clone,
{
    fn absorb_first(&mut self, operation: &mut SystemOp<T>, _: &Self) {
        match operation {
            SystemOp::New(plugin_name, plugin) => {
                self.plugins.insert(plugin_name.to_owned(), plugin.clone());
            }

            SystemOp::Remove(plugin_name) => {
                self.plugins.remove(&plugin_name.to_owned());
            }
        }
    }

    fn sync_with(&mut self, first: &Self) {
        *self = first.clone();
    }

    fn drop_first(self: Box<Self>) {}
}

#[derive(Debug)]
pub struct PluginSystemWriter<T: Plugin<WasmPlugin> + Clone>(
    pub WriteHandle<PluginSystem<T>, SystemOp<T>>,
);

impl<T> PluginSystemWriter<T>
where
    T: Plugin<WasmPlugin> + Clone,
{
    /// Adds the new plugin to the plugins list(map)
    pub fn add(&mut self, plugin: T) -> &Self {
        self.0.append(SystemOp::New(plugin.metadata().name, plugin));

        self
    }

    /// Removes the plugin from the plugins list(map)
    pub fn remove(&mut self, plugin: T) -> &Self {
        // TODO: handle the option
        self.0.append(SystemOp::Remove(plugin.metadata().name));

        self
    }

    /// Commits the changes
    pub fn publish(&mut self) -> &Self {
        self.0.publish();

        self
    }
}

impl PluginSystemWriter<PluginBuilder> {
    pub fn add_from_config(
        &mut self,
        wasm_as_bytes: Vec<u8>,
        config: PluginConfig<PluginMetadata>,
    ) -> Result<(), ManagerError> {
        // Build the plugin with pluginBuilder
        let new_plugin = PluginBuilder::new(config, wasm_as_bytes);

        self.add(new_plugin);

        Ok(())
    }
}

pub struct PluginSystemReader<T: Plugin<WasmPlugin> + Clone>(
    // We need to use the readhandlefactory bequase we need to share the data
    // across multiple threads
    pub ReadHandleFactory<PluginSystem<T>>,
);

impl<T> PluginSystemReader<T>
where
    T: Plugin<WasmPlugin> + Clone,
{
    pub fn get(&self, name: &String) -> Option<T> {
        self.0.handle().enter().unwrap().plugins.get(name).cloned()
    }
}

pub type BuilderReader = PluginSystemReader<PluginBuilder>;
