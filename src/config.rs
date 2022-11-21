use serde::{Deserialize, Serialize};
use std::fs::File;
use std::result::Result;
use std::io::{Error};
use std::path::{Path, PathBuf};
use cached::{Cached, cached_result};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Location {
    pub name: String,
    pub id: String,
    pub priority: i64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    pub schema_version: String,
    pub location: Vec<Location>,
}

cached_result! {
    CONFIG_CACHE: cached::UnboundCache<PathBuf, Config> = cached::UnboundCache::new();
    fn read_config_cached(path: PathBuf) -> Result<Config, Error> = {
        let config_file = File::open(path)?;
        let config: Config = serde_json::from_reader(config_file)?;
        Ok(config)
    }
}

pub fn read_config(root_path: &str) -> Result<Config, Error> {
    let path = Path::new(root_path)
        .join(".outpack")
        .join("config.json");
    read_config_cached(path)
}

// to be used when changing config settings
pub fn clear_cache() {
    CONFIG_CACHE.lock().unwrap().cache_clear();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_clear_cache() {
        let _cfg = read_config("tests/example");
        let _cached_cfg = read_config("tests/example");
        assert_eq!(CONFIG_CACHE.lock().unwrap().cache_hits().unwrap(), 1);

        clear_cache();

        let _fresh_cfg = read_config("tests/example");
        // should still only be 1 hit, since the last read should have been from disk
        assert_eq!(CONFIG_CACHE.lock().unwrap().cache_hits().unwrap(), 1);
    }
}
