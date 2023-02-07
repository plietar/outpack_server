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

#[allow(non_camel_case_types)]
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum HashAlgorithm {
    md5,
    sha1,
    sha256,
    sha384,
    sha512,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Core {
    pub path_archive: Option<String>,
    pub use_file_store: bool,
    pub hash_algorithm: HashAlgorithm,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub schema_version: String,
    pub location: Vec<Location>,
    pub core: Core,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_read_config() {
        let cfg = read_config("tests/example").unwrap();
        assert_eq!(cfg.core.hash_algorithm, HashAlgorithm::sha256);
        assert!(cfg.core.use_file_store);
        assert!(cfg.core.path_archive.is_none());
    }
}

