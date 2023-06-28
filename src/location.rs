use serde::{Deserialize, Serialize};
use std::{fs, io};
use std::ffi::{OsString};
use std::fs::{DirEntry};
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;
use cached::cached_result;
use cached::instant::SystemTime;
use crate::config::Location;

use super::config;
use super::utils;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LocationEntry {
    pub packet: String,
    pub time: f64,
    pub hash: String,
    pub schema_version: String,
}

cached_result! {
    ENTRY_CACHE: cached::UnboundCache<PathBuf, LocationEntry> = cached::UnboundCache::new();
    fn read_entry(path: PathBuf) -> io::Result<LocationEntry> = {
        let file = fs::File::open(path)?;
        let entry: LocationEntry = serde_json::from_reader(file)?;
        Ok(entry)
    }
}

fn get_priority(location_config: &[Location], entry: &DirEntry) -> i64 {
    let id = entry.file_name();
    location_config.iter()
        .find(|l| OsString::from(&l.id) == id)
        .map(|l| l.priority).unwrap()
}

pub fn read_location(path: PathBuf) -> io::Result<Vec<LocationEntry>> {
    let mut packets = fs::read_dir(path)?
        .filter_map(|e| e.ok())
        .filter(|e| utils::is_packet(&e.file_name()))
        .map(|entry| read_entry(entry.path()))
        .collect::<io::Result<Vec<LocationEntry>>>()?;

    packets.sort_by(|a, b| a.packet.cmp(&b.packet));

    Ok(packets)
}

pub fn read_locations(root_path: &str) -> io::Result<Vec<LocationEntry>> {
    let path = Path::new(root_path)
        .join(".outpack")
        .join("location");

    let location_config = config::read_config(root_path)?.location;

    let mut locations_sorted = fs::read_dir(path)?
        .filter_map(|r| r.ok())
        .collect::<Vec<DirEntry>>();

    locations_sorted.sort_by_key(|a| get_priority(&location_config, a));

    let packets = locations_sorted
        .iter()
        .map(|entry| read_location(entry.path()))
        // collect any errors at this point into a single result
        .collect::<io::Result<Vec<Vec<LocationEntry>>>>()?
        .into_iter()
        .flatten()
        .collect();

    Ok(packets)
}

pub fn get_local_location_id(root: &str) -> io::Result<String> {
    let config = config::read_config(root)?;
    Ok(config.location.iter().find(|l| l.name == "local")
        .ok_or(io::Error::new(io::ErrorKind::InvalidData, "No local location"))?.id.clone())
}

pub fn mark_packet_known(packet_id: &str, location_id: &str, hash: &str, time: SystemTime, root: &str) -> io::Result<()> {
    let schema_version = config::read_config(root)?.schema_version;
    let entry = LocationEntry {
        packet: String::from(packet_id),
        time: time.duration_since(UNIX_EPOCH).unwrap().as_millis() as f64,
        hash: String::from(hash),
        schema_version,
    };

    let path = Path::new(root)
        .join(".outpack")
        .join("location")
        .join(location_id)
        .join(packet_id);

    fs::File::create(&path)?;
    let json = serde_json::to_string(&entry)?;
    fs::write(path, json)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn packets_ordered_by_location_priority_then_id() {
        let entries = read_locations("tests/example").unwrap();
        assert_eq!(entries[0].packet, "20170818-164847-7574883b");
        assert_eq!(entries[1].packet, "20170818-164830-33e0ab01");
        assert_eq!(entries[2].packet, "20180818-164043-7cdcde4b");
    }
}
