use serde::{Deserialize, Serialize};
use std::fs::File;
use std::result::Result;
use std::io::{Error};
use std::path::{Path};

#[derive(Serialize, Deserialize, Debug)]
pub struct Location {
    pub name: String,
    pub id: String,
    pub priority: i64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub schema_version: String,
    pub location: Vec<Location>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Root {
    pub schema_version: String,
}

impl Root {
    pub fn new(schema_version: String) -> Root {
        Root {
            schema_version
        }
    }
}

pub fn read_config(root_path: &str) -> Result<Config, Error> {
    let path = Path::new(root_path)
        .join(".outpack")
        .join("config.json");
    let config_file = File::open(path)?;
    let config: Config = serde_json::from_reader(config_file)?;
    Ok(config)
}
