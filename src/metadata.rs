use std::{fs, io};
use std::path::{Path, PathBuf};

use super::config;
use super::hash;

fn get_metadata_file(root_path: &str, id: &str) -> io::Result<PathBuf> {
    let path = Path::new(root_path)
        .join(".outpack")
        .join("metadata")
        .join(id);

    return if !path.exists() {
        Err(io::Error::new(io::ErrorKind::NotFound,
                           format!("packet with id '{}' does not exist", id)))
    } else {
        Ok(path)
    };
}

pub fn get_metadata(root_path: &str, id: &str) -> io::Result<serde_json::Value> {
    let path = get_metadata_file(root_path, id)?;
    let file = fs::File::open(&path)?;
    let packet = serde_json::from_reader(file)?;
    Ok(packet)
}

pub fn get_metadata_text(root_path: &str, id: &str) -> io::Result<String> {
    let path = get_metadata_file(root_path, id)?;
    fs::read_to_string(path)
}

pub fn get_ids_digest(root_path: &str) -> io::Result<String> {
    let core_config = config::read_config(root_path)?.core;

    let path = Path::new(root_path)
        .join(".outpack")
        .join("metadata");

    let ids = fs::read_dir(path)?
        .filter_map(|r| r.ok())
        .map(|e| e.file_name().into_string())
        .filter_map(|r| r.ok())
        .collect::<Vec<String>>().join("");

    Ok(hash::hash_data(ids, core_config.hash_algorithm))
}

#[cfg(test)]
mod tests {
    use sha2::{Sha256, Digest};
    use super::*;

    #[test]
    fn can_get_packet() {
        let _packet = get_metadata("tests/example", "20170818-164847-7574883b")
            .unwrap();
    }

    #[test]
    fn can_get_ids_digest() {
        let digest = get_ids_digest("tests/example")
            .unwrap();
        let expected = format!("sha256:{:x}",
                               Sha256::new()
                                   .chain_update("20170818-164847-7574883c20170818-164847-7574883b")
                                   .finalize());
        assert_eq!(digest, expected);
    }
}
