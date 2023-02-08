use std::{fs, io};
use std::path::{Path, PathBuf};
use std::str::FromStr;
use crate::config::HashAlgorithm;

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

fn get_sorted_id_string(mut ids: Vec<String>) -> String {
    ids.sort_by(|a, b| a.cmp(b));
    ids.join("")
}

pub fn get_ids_digest(root_path: &str, alg_name: Option<String>) -> io::Result<String> {
    let hash_algorithm = match alg_name {
        None => config::read_config(root_path)?.core.hash_algorithm,
        Some(name) => HashAlgorithm::from_str(&name)
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidData,
                                        format!("algorithm {} not found", name)))?
    };

    let path = Path::new(root_path)
        .join(".outpack")
        .join("metadata");

    let ids = fs::read_dir(path)?
        .filter_map(|r| r.ok())
        .map(|e| e.file_name().into_string())
        .filter_map(|r| r.ok())
        .collect::<Vec<String>>();

    let id_string = get_sorted_id_string(ids);

    Ok(hash::hash_data(id_string, hash_algorithm))
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
    fn ids_are_sorted() {
        let ids = vec![String::from("20180818-164847-7574883b"),
                       String::from("20170818-164847-7574883b"),
                       String::from("20170819-164847-7574883b"),
                       String::from("20170819-164847-7574883a")];
        let id_string = get_sorted_id_string(ids);
        assert_eq!(id_string, "20170818-164847-7574883b20170819-164847-7574883a20170819-164847-7574883b20180818-164847-7574883b")
    }

    #[test]
    fn can_get_ids_digest_with_config_alg() {
        let digest = get_ids_digest("tests/example", None)
            .unwrap();
        let expected = format!("sha256:{:x}",
                               Sha256::new()
                                   .chain_update("20170818-164847-7574883b20170818-164847-7574883c")
                                   .finalize());
        assert_eq!(digest, expected);
    }

    #[test]
    fn can_get_ids_digest_with_given_alg() {
        let digest = get_ids_digest("tests/example", Some(String::from("md5")))
            .unwrap();
        let expected = format!("md5:{:x}",
                               md5::compute("20170818-164847-7574883b20170818-164847-7574883c"));
        assert_eq!(digest, expected);
    }
}
