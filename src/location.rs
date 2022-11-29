use serde::{Deserialize, Serialize};
use std::{fs, io};
use std::ffi::{OsString};
use std::fs::{DirEntry};
use std::path::{Path, PathBuf};
use regex::Regex;
use cached::cached_result;
use crate::config::Location;

use super::config;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LocationEntry {
    pub packet: String,
    pub time: f64,
    pub hash: String,
}

const ID_REG: &str = "^([0-9]{8}-[0-9]{6}-[[:xdigit:]]{8})$";
cached_result! {
    ENTRY_CACHE: cached::UnboundCache<PathBuf, LocationEntry> = cached::UnboundCache::new();
    fn read_entry(path: PathBuf) -> io::Result<LocationEntry> = {
        let file = fs::File::open(path)?;
        let entry: LocationEntry = serde_json::from_reader(file)?;
        Ok(entry)
    }
}

fn is_packet(name: &OsString, reg: &Regex) -> bool {
    let o = name.to_str();
    o.map_or(false, |s| reg.is_match(s))
}

fn get_priority(location_config: &Vec<Location>, entry: &DirEntry) -> i64 {
    let id = entry.file_name();
    location_config.into_iter()
        .find(|l| OsString::from(&l.id) == id)
        .map(|l| l.priority).unwrap()
}

pub fn read_location(path: PathBuf, reg: &Regex) -> io::Result<Vec<LocationEntry>> {
    let mut packets = fs::read_dir(path)?
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| is_packet(&e.file_name(), &reg))
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

    let mut locations_sorted = fs::read_dir(&path)?
        .into_iter()
        .filter_map(|r| r.ok())
        .collect::<Vec<DirEntry>>();

    locations_sorted.sort_by(|a, b| get_priority(&location_config, a).cmp(&get_priority(&location_config, b)));

    let reg = Regex::new(ID_REG).unwrap();

    let packets = locations_sorted
        .into_iter()
        .map(|entry| read_location(entry.path(), &reg))
        .into_iter()
        // collect any errors at this point into a single result
        .collect::<io::Result<Vec<Vec<LocationEntry>>>>()?
        .into_iter()
        .flatten()
        .collect();

    Ok(packets)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_detect_packet_id() {
        let reg = Regex::new(ID_REG).unwrap();
        assert_eq!(is_packet(&OsString::from("1234"), &reg), false);
        assert_eq!(is_packet(&OsString::from("20170818-164830-33e0ab01"), &reg), true);
    }

    #[test]
    fn packets_ordered_by_location_priority_then_id() {
        let entries = read_locations("tests/example").unwrap();
        assert_eq!(entries[0].packet, "20170818-164847-7574883b");
        assert_eq!(entries[1].packet, "20170818-164043-7cdcde4b");
        assert_eq!(entries[2].packet, "20170818-164830-33e0ab01");
    }
}
