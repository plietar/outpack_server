use serde::{Deserialize, Serialize};
use std::fs::File;
use std::result::Result;
use std::io::{Error};
use std::path::{Path};

use crate::hash::{HashAlgorithm};

#[derive(Serialize, Deserialize, Debug)]
pub struct Location {
    // Practically, doing anything with locations (therefore needing
    // access to the "type" and "args" fields) is going to require we
    // know how to deserialise into a union type; for example
    // https://stackoverflow.com/q/66964692
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Core {
    pub hash_algorithm: HashAlgorithm,
    pub path_archive: Option<String>,
    pub use_file_store: bool,
    pub require_complete_tree: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub location: Vec<Location>,
    pub core: Core,
}

pub fn read_config(root_path: &str) -> Result<Config, Error> {
    let path = Path::new(root_path)
        .join(".outpack")
        .join("config.json");
    let config_file = File::open(path)?;
    let config: Config = serde_json::from_reader(config_file)?;
    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_read_config() {
        let cfg = read_config("tests/example").unwrap();
        assert_eq!(cfg.core.hash_algorithm, HashAlgorithm::Sha256);
        assert!(cfg.core.use_file_store);
        assert!(cfg.core.require_complete_tree);
        assert!(cfg.core.path_archive.is_none());
    }
}
