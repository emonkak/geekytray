use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::command::Command;
use crate::ui::{Key, Modifiers};

#[derive(Debug)]
pub struct KeyMappingInterInterpreter {
    command_table: HashMap<(Key, Modifiers), Vec<Command>>,
}

impl KeyMappingInterInterpreter {
    pub fn new(key_mappings: Vec<KeyMapping>) -> Self {
        let mut command_table: HashMap<(Key, Modifiers), Vec<Command>> = HashMap::new();
        for key_mapping in key_mappings {
            command_table.insert(
                (key_mapping.key, key_mapping.modifiers),
                key_mapping.commands.clone(),
            );
        }
        Self { command_table }
    }

    pub fn eval(&self, key: Key, modifiers: Modifiers) -> &[Command] {
        self.command_table
            .get(&(key, modifiers))
            .map(|commands| commands.as_slice())
            .unwrap_or_default()
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct KeyMapping {
    key: Key,
    #[serde(default = "Modifiers::none")]
    modifiers: Modifiers,
    commands: Vec<Command>,
}

impl KeyMapping {
    pub fn new(key: impl Into<Key>, modifiers: Modifiers, commands: Vec<Command>) -> Self {
        Self {
            key: key.into(),
            modifiers,
            commands,
        }
    }
}
