use anyhow::{anyhow, Result};
use extism::{Plugin, Manifest, Wasm};
use std::collections::HashMap;
use log::info;

pub struct PluginManager {
    plugins: HashMap<String, Plugin>,
}

impl PluginManager {
    pub fn new() -> Self {
        Self {
            plugins: HashMap::new(),
        }
    }

    pub fn load_plugin(&mut self, name: &str, wasm_bytes: &[u8]) -> Result<()> {
        info!("Loading plugin: {}", name);
        let wasm = Wasm::data(wasm_bytes);
        let manifest = Manifest::new([wasm]);
        let plugin = Plugin::new(&manifest, [], true)?;
        
        self.plugins.insert(name.to_string(), plugin);
        Ok(())
    }

    pub fn trigger_event(&mut self, plugin_name: &str, event_name: &str, input: &[u8]) -> Result<Vec<u8>> {
        if let Some(plugin) = self.plugins.get_mut(plugin_name) {
            let output: &[u8] = plugin.call(event_name, input)?;
            Ok(output.to_vec())
        } else {
            Err(anyhow!("Plugin not found: {}", plugin_name))
        }
    }
}

impl Default for PluginManager {
    fn default() -> Self {
        Self::new()
    }
}
