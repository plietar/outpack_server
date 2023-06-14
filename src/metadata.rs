use serde::{Deserialize, Serialize};
use std::{fs, io};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use cached::cached_result;
use crate::config::HashAlgorithm;
use crate::location::read_locations;
use crate::utils::is_packet_str;

use super::config;
use super::hash;
use super::utils;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Packet {
    pub id: String,
    pub name: String,
    pub custom: Option<serde_json::Value>,
    pub parameters: Option<serde_json::Value>,
}

cached_result! {
    ENTRY_CACHE: cached::UnboundCache<PathBuf, Packet> = cached::UnboundCache::new();
    fn read_entry(path: PathBuf) -> io::Result<Packet> = {
        let file = fs::File::open(path)?;
        let entry: Packet = serde_json::from_reader(file)?;
        Ok(entry)
    }
}

fn get_metadata_file(root_path: &str, id: &str) -> io::Result<PathBuf> {
    let path = Path::new(root_path)
        .join(".outpack")
        .join("metadata")
        .join(id);

    if !path.exists() {
        Err(io::Error::new(io::ErrorKind::NotFound,
                           format!("packet with id '{}' does not exist", id)))
    } else {
        Ok(path)
    }
}

pub fn get_metadata_from_date(root_path: &str, from: Option<f64>) -> io::Result<Vec<Packet>> {
    let path = Path::new(root_path)
        .join(".outpack")
        .join("metadata");

    let packets = fs::read_dir(path)?
        .filter_map(|e| e.ok())
        .filter(|e| utils::is_packet(&e.file_name()));

    let mut packets = match from {
        None => packets.map(|entry| read_entry(entry.path()))
            .collect::<io::Result<Vec<Packet>>>()?,
        Some(time) => {
            let location_meta = read_locations(root_path)?;
            packets.filter(
                |entry| location_meta.iter()
                    .find(|&e| e.packet == entry.file_name().into_string().unwrap())
                    .map_or(false, |e| e.time > time)
            )
                .map(|entry| read_entry(entry.path()))
                .collect::<io::Result<Vec<Packet>>>()?
        }
    };

    packets.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(packets)
}

pub fn get_metadata_by_id(root_path: &str, id: &str) -> io::Result<serde_json::Value> {
    let path = get_metadata_file(root_path, id)?;
    let file = fs::File::open(path)?;
    let packet = serde_json::from_reader(file)?;
    Ok(packet)
}

pub fn get_metadata_text(root_path: &str, id: &str) -> io::Result<String> {
    let path = get_metadata_file(root_path, id)?;
    fs::read_to_string(path)
}

fn get_sorted_id_string(mut ids: Vec<String>) -> String {
    ids.sort();
    ids.join("")
}

pub fn get_ids_digest(root_path: &str, alg_name: Option<String>) -> io::Result<String> {
    let hash_algorithm = match alg_name {
        None => config::read_config(root_path)?.core.hash_algorithm,
        Some(name) => HashAlgorithm::from_str(&name)
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidData,
                                        format!("algorithm {} not found", name)))?
    };

    let ids = get_ids(root_path, None)?;
    let id_string = get_sorted_id_string(ids);

    Ok(hash::hash_data(id_string, hash_algorithm))
}

pub fn get_ids(root_path: &str, unpacked: Option<bool>) -> io::Result<Vec<String>> {
    let dir_name = match unpacked {
        None => "metadata",
        Some(unpacked) => {
            if unpacked { "unpacked" } else { "metadata" }
        }
    };
    let path = Path::new(root_path)
        .join(".outpack")
        .join(dir_name);

    Ok(fs::read_dir(path)?
        .filter_map(|r| r.ok())
        .map(|e| e.file_name().into_string())
        .filter_map(|r| r.ok())
        .collect::<Vec<String>>())
}

pub fn get_valid_id(id: &str) -> io::Result<String> {
    let s = String::from(id.trim());
    if is_packet_str(&s) {
        Ok(s)
    } else {
        Err(io::Error::new(io::ErrorKind::InvalidInput,
                           format!("Invalid packet id '{}'", id)))
    }
}

pub fn get_missing_ids(root_path: &str, wanted: &str, unpacked: Option<bool>) -> io::Result<Vec<String>> {
    let known: HashSet<String> = get_ids(root_path, unpacked)?.into_iter().collect();
    let wanted: HashSet<String> = wanted.split(',')
        .map(get_valid_id)
        .collect::<io::Result<HashSet<String>>>()?;
    Ok(wanted.difference(&known).cloned().collect::<Vec<String>>())
}

#[cfg(test)]
mod tests {
    use sha2::{Sha256, Digest};
    use super::*;

    #[test]
    fn can_get_packets_from_date() {
        let all_packets = get_metadata_from_date("tests/example", None)
            .unwrap();
        assert_eq!(all_packets.len(), 3);
        let recent_packets = get_metadata_from_date("tests/example",
                                                    Some(1662480556 as f64))
            .unwrap();
        assert_eq!(recent_packets.len(), 1);
        assert_eq!(recent_packets.first().unwrap().id, "20170818-164847-7574883b");

        let recent_packets = get_metadata_from_date("tests/example",
                                                    Some(1662480555 as f64))
            .unwrap();
        assert_eq!(recent_packets.len(), 3);
    }

    #[test]
    fn can_get_packet() {
        let _packet = get_metadata_by_id("tests/example", "20180818-164043-7cdcde4b")
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
        let dat = "20170818-164830-33e0ab0120170818-164847-7574883b20180818-164043-7cdcde4b";
        let expected = format!("sha256:{:x}",
                               Sha256::new()
                                   .chain_update(dat)
                                   .finalize());
        assert_eq!(digest, expected);
    }

    #[test]
    fn can_get_ids_digest_with_given_alg() {
        let digest = get_ids_digest("tests/example", Some(String::from("md5")))
            .unwrap();
        let dat = "20170818-164830-33e0ab0120170818-164847-7574883b20180818-164043-7cdcde4b";
        let expected = format!("md5:{:x}",
                               md5::compute(dat));
        assert_eq!(digest, expected);
    }

    #[test]
    fn can_get_ids() {
        let ids = get_ids("tests/example", None)
            .unwrap();
        assert_eq!(ids.len(), 3);
        assert!(ids.iter().any(|e| e == "20170818-164830-33e0ab01"));
        assert!(ids.iter().any(|e| e == "20170818-164847-7574883b"));
        assert!(ids.iter().any(|e| e == "20180818-164043-7cdcde4b"));
    }

    #[test]
    fn can_get_unpacked_ids() {
        let ids = get_ids("tests/example", Some(true))
            .unwrap();
        assert_eq!(ids.len(), 1);
        assert!(ids.iter().any(|e| e == "20170818-164830-33e0ab01"));
    }

    #[test]
    fn can_get_missing_ids() {
        let ids = get_missing_ids("tests/example",
                                  "20180818-164043-7cdcde4b,20170818-164830-33e0ab02",
                                  None)
            .unwrap();
        assert_eq!(ids.len(), 1);
        assert!(ids.iter().any(|e| e == "20170818-164830-33e0ab02"));

        // check whitespace insensitivity
        let ids = get_missing_ids("tests/example",
                                  "20180818-164043-7cdcde4b, 20170818-164830-33e0ab02",
                                  None)
            .unwrap();
        assert_eq!(ids.len(), 1);
        assert!(ids.iter().any(|e| e == "20170818-164830-33e0ab02"));
    }

    #[test]
    fn can_get_missing_unpacked_ids() {
        let ids = get_missing_ids("tests/example",
                                  "20170818-164830-33e0ab01,20170818-164830-33e0ab02",
                                  Some(true))
            .unwrap();
        assert_eq!(ids.len(), 1);
        assert!(ids.iter().any(|e| e == "20170818-164830-33e0ab02"));
    }

    #[test]
    fn bad_ids_raise_error() {
        let res = get_missing_ids("tests/example",
                                  "20180818-164043-7cdcde4b,20170818-164830-33e0ab0",
                                  None).map_err(|e| e.kind());
        assert_eq!(Err(io::ErrorKind::InvalidInput), res);
    }
}
