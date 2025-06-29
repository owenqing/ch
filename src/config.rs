use std::collections::HashMap;
use anyhow::Result;
use serde::Deserialize;
use std::fs::File;
use std::io::Read;

#[derive(Deserialize)]
pub struct Config {
    pub groups: HashMap<String, Group>,
}

#[derive(Deserialize)]
pub struct Group {
    pub connections: HashMap<String, String>,
}

pub fn load_config(path: &str) -> Result<Config> {
    let mut file = File::open(path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    let config: Config = toml::from_str(&contents)?;
    Ok(config)
} 