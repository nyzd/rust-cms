use wasmer::{Exports, Imports, Instance, Module, Store};

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
    pub source: Vec<u8>,
    pub store: Store,
    pub instance: Option<Instance>,
}

impl WasmPlugin {
    /// Creates a new wasm plugin object with a store and instance
    pub fn new(source: Vec<u8>) -> Self {
        let store = Store::default();

        Self {
            store,
            source,
            instance: None,
        }
    }

    pub fn init_instance(&mut self, imports: Imports) -> Result<(), WasmError> {
        let Ok(module) = Module::new(&self.store, self.source.clone()) else {
            return Err(WasmError::Compile("Cant Compile the wasm file!".to_string()));
        };

        let instance = Instance::new(&mut self.store, &module, &imports).unwrap();
        self.instance = Some(instance);

        Ok(())
    }

    /// Returns the exports name
    pub fn export_names(&self) -> Vec<String> {
        let exports = &self.instance.clone().unwrap().exports;

        exports
            .into_iter()
            .map(|(x, _y)| x.clone())
            .collect::<Vec<String>>()
    }
}

#[cfg(test)]
mod tests {

    use wasmer::Value;

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

        let mut wasm_plugin = WasmPlugin::new(&source.as_bytes().to_vec()).unwrap();

        assert_eq!(wasm_plugin.export_names(), vec!["add_one"]);

        let function = wasm_plugin
            .instance
            .exports
            .get_function("add_one")
            .unwrap();
        let result = function.call(&mut wasm_plugin.store, &[]).unwrap();

        assert_eq!(result.get(0).unwrap(), &Value::I32(2));
    }
}
