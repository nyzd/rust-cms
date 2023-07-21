use actix_web::web;
use plugin_manager::{
    config::PluginAbiParamType,
    manager::{BuilderReader, Plugin},
    wasmer::{imports, Exports, Extern, Function},
};
use serde::Deserialize;

use crate::error::router_error::RouterError;

#[derive(Deserialize, Debug, Clone)]
pub struct PluginCall {
    /// Name of plugin
    pub name: String,

    /// Plugin function name
    pub function_name: String,
}

/// Runs the plugin function and returns result as
/// response
pub async fn run_plugin_function(
    plugin_reader: web::Data<BuilderReader>,
    req_json: web::Json<PluginCall>,
) -> Result<web::Json<String>, RouterError> {
    // Get the reader
    let reader = plugin_reader.into_inner();
    let req_json = req_json.into_inner();

    let Some(plugin) = reader.get(&req_json.name) else {
        return Err(RouterError::NotFound(format!("Plugin name {} not found!", &req_json.name)));
    };

    let mut plugin_wasm = plugin.build().unwrap();
    let imports = imports! {
        "cms" => {
            "log" => Function::new_typed(&mut plugin_wasm.store, || println!("HAHA")),
        }
    };

    plugin_wasm.init_instance(imports).unwrap();

    if !plugin_wasm
        .export_names()
        .into_iter()
        .any(|i| i == req_json.function_name)
    {
        return Err(RouterError::NotFound(format!(
            "Function name {} not found!",
            &req_json.function_name
        )));
    }

    let instance = plugin_wasm.instance.clone().unwrap();

    let function = instance
        .exports
        .get_function(&req_json.function_name)
        .unwrap();

    let function_result = function.call(&mut plugin_wasm.store, &[]).unwrap();

    let result = match plugin
        .abi()
        .functions
        .into_iter()
        .find(|f| f.name == req_json.function_name)
        .unwrap()
        .result
        .ty
    {
        PluginAbiParamType::String => {
            let memory = instance
                .exports
                .get_memory("memory")
                .unwrap();

            // Not a good way
            let buf = memory.view(&mut plugin_wasm.store).copy_to_vec().unwrap();

            let buf_iter = buf
                .into_iter()
                .skip(function_result[0].unwrap_i32() as usize)
                .take_while(|n| *n != 0)
                //.map(|n| char::from(n))
                .collect::<Vec<u8>>();

            String::from_utf8(buf_iter).unwrap()
        }

        PluginAbiParamType::Number => function_result[0].to_string(),
    };

    Ok(web::Json(result))
}
