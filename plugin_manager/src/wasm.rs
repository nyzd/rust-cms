use wasmer::{imports, Instance, Module, Store, Value};

#[derive(Debug)]
pub enum WasmError {
    Compile(String),
    Instance(String),
    Export(String),
    Runtime(String),
}

/// Get the wasm source and compile it
/// and build and return the Plugin
#[derive(Debug)]
pub struct WasmPlugin {
    store: Store,
    instance: Instance,
}

impl WasmPlugin {
    /// Creates a new wasm plugin object with a store and instance
    pub fn new(source: String) -> Result<Self, WasmError> {
        let mut store = Store::default();
        let Ok(module) = Module::new(&store, &source) else {
            return Err(WasmError::Compile("Cant Compile the wasm file!".to_string()));
        };
        let import_object = imports! {};
        // TODO: is it okey to use variable store and not the self.store ?
        let Ok(instance) = Instance::new(&mut store, &module, &import_object) else {
            return Err(WasmError::Instance("Cant create a instance!".to_string()));
        };

        Ok(Self { store, instance })
    }

    /// Returns the exports name
    pub fn export_names(&self) -> Vec<String> {
        let exports = &self.instance.exports;

        exports
            .into_iter()
            .map(|(x, _y)| x.clone())
            .collect::<Vec<String>>()
    }

    /// Runs the wasm function
    pub fn run_func(
        &mut self,
        function_name: String,
        params: Vec<Value>,
    ) -> Result<Box<[Value]>, WasmError> {
        let exports = &self.instance.exports;

        let Ok(function) = exports.get_function(&function_name) else {
            return Err(WasmError::Export(format!("Cant get the function {}", &function_name)));
        };

        let Ok(result) = function.call(&mut self.store, &params) else {
            return Err(WasmError::Runtime(format!("Cant run the function {}", &function_name)));
        };

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use std::vec;

    use super::*;

    #[test]
    fn test_wasm_plugin() {
        let source = r#"(module
      (type $t0 (func (param i32) (result i32)))
      (func $add_one (export "add_one") (type $t0) (param $p0 i32) (result i32)
        get_local $p0
        i32.const 1
        i32.add))"#
            .to_string();

        let mut wasm_plugin = WasmPlugin::new(source).unwrap();

        assert_eq!(wasm_plugin.export_names(), vec!["add_one"]);

        let result = wasm_plugin
            .run_func("add_one".to_string(), vec![Value::I32(1)])
            .unwrap();

        assert_eq!(result.get(0).unwrap(), &Value::I32(2));
    }
}
