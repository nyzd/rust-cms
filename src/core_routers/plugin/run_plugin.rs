use actix_web::web;
use plugin_manager::manager::{BuilderReader, Plugin};
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
) -> Result<web::Json<Vec<String>>, RouterError> {
    // Get the reader
    let reader = plugin_reader.into_inner();
    let req_json = req_json.into_inner();

    let Some(plugin) = reader.get(&req_json.name) else {
        return Err(RouterError::NotFound(format!("Plugin name {} not found!", &req_json.name)));
    };

    let Ok(mut plugin_wasm) = plugin.build() else {
        return Err(RouterError::InternalError);
    };

    if plugin_wasm
        .export_names()
        .into_iter()
        .any(|i| i != req_json.function_name)
    {
        return Err(RouterError::NotFound(format!(
            "Function name {} not found!",
            &req_json.function_name
        )));
    }

    // TODO: return error with the description
    let Ok(func_result)= plugin_wasm.run_func(req_json.function_name, vec![]) else {
        return Err(RouterError::InternalError);
    };

    // TODO: return the better result
    //       perefered type is a json result
    let result = func_result
        .into_iter()
        .map(|val| val.to_string())
        .collect::<Vec<String>>();

    Ok(web::Json(result))
}
