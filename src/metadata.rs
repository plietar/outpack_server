use std::{fs, io};
use std::path::{Path, PathBuf};
use md5;
use sha1::{Sha1};
use sha2::{Sha256, Sha512, Sha384, Digest};
use crate::config::HashAlgorithm;

use super::config;

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

    let hash = match core_config.hash_algorithm {
        HashAlgorithm::md5 => format!("md5:{:x}", md5::compute(ids)),
        HashAlgorithm::sha1 => format!("sha1:{:x}", Sha1::new()
            .chain_update(ids)
            .finalize()),
        HashAlgorithm::sha256 => format!("sha256:{:x}", Sha256::new()
            .chain_update(ids)
            .finalize()),
        HashAlgorithm::sha384 => format!("sha384:{:x}", Sha384::new()
            .chain_update(ids)
            .finalize()),
        HashAlgorithm::sha512 => format!("sha512:{:x}", Sha512::new()
            .chain_update(ids)
            .finalize()),
    };

    Ok(hash)
}

#[cfg(test)]
mod tests {
    use crate::config::HashAlgorithm::sha256;
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
