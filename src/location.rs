use serde::{Deserialize, Serialize};
use std::{fs, io};
use std::ffi::{OsString};
use std::fs::{DirEntry};
use std::path::{Path, PathBuf};
use cached::cached_result;
use cached::instant::SystemTime;
use crate::config::Location;
use crate::utils::time_as_num;

use super::config;
use super::utils;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
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

fn get_order(location_config: &[Location], entry: &DirEntry) -> usize {
    let id = entry.file_name();
    location_config.iter()
        .position(|l| OsString::from(&l.id) == id)
        .unwrap()
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

    locations_sorted.sort_by_key(|a| get_order(&location_config, a));

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

pub fn get_local_location_id(root_path: &str) -> io::Result<String> {
    let location = config::read_config(root_path)?
        .location
        .iter()
        .find(|loc| loc.name == "local")
        .unwrap() // every outpack configuration must have this.
        .id.clone();
    Ok(location)
}

pub fn mark_packet_known(packet_id: &str, location_id: &str, hash: &str, time: SystemTime, root: &str) -> io::Result<()> {
    let schema_version = config::read_config(root)?.schema_version;
    let entry = LocationEntry {
        packet: String::from(packet_id),
        time: time_as_num(time),
        hash: String::from(hash),
        schema_version,
    };

    let location_path = Path::new(root)
        .join(".outpack")
        .join("location")
        .join(location_id);

    fs::create_dir_all(&location_path)?;
    let path = location_path.join(packet_id);
    if !path.exists() {
        fs::File::create(&path)?;
        let json = serde_json::to_string(&entry)?;
        fs::write(path, json)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, SystemTime};
    use crate::test_utils::tests::get_temp_outpack_root;
    use super::*;

    #[test]
    fn packets_ordered_by_location_order_then_id() {
        let entries = read_locations("tests/example").unwrap();
        assert_eq!(entries[0].packet, "20170818-164847-7574883b");
        assert_eq!(entries[1].packet, "20170818-164830-33e0ab01");
        assert_eq!(entries[2].packet, "20180220-095832-16a4bbed");
        assert_eq!(entries[3].packet, "20180818-164043-7cdcde4b");
    }

    #[test]
    fn can_find_local_id() {
        assert_eq!(get_local_location_id("tests/example").unwrap(), "be7a7bcb");
    }

    #[test]
    fn can_mark_known() {
        let root = get_temp_outpack_root();
        let loc_a = Path::new(root.as_path()).join(".outpack/location/ae7a7bcb");
        let entries_a = read_location(loc_a).unwrap();
        let entry_a = entries_a.first().unwrap();
        let loc_b = Path::new(root.as_path()).join(".outpack/location/be7a7bcb");
        let entries_b = read_location(loc_b.clone()).unwrap();
        assert!(entries_b.iter().find(|e| e.packet == entry_a.packet).is_none());
        let now = SystemTime::now();
        mark_packet_known(&entry_a.packet, "be7a7bcb", &entry_a.hash,
                          SystemTime::now(), root.as_path().to_str().unwrap()).unwrap();
        let entries_b = read_location(loc_b).unwrap();
        let res = entries_b.iter().find(|e| e.packet == entry_a.packet).unwrap();
        assert_eq!(res.time, time_as_num(now));
        assert_eq!(res.packet, entry_a.packet);
        assert_eq!(res.hash, entry_a.hash);
        assert_eq!(res.schema_version, entry_a.schema_version);
    }

    #[test]
    fn marking_known_does_not_overwrite() {
        let root = get_temp_outpack_root();
        let loc_a = Path::new(root.as_path()).join(".outpack/location/ae7a7bcb");
        let entries_a = read_location(loc_a).unwrap();
        let entry_a = entries_a.first().unwrap();
        let now = SystemTime::now();
        mark_packet_known(&entry_a.packet, "be7a7bcb", &entry_a.hash,
                          SystemTime::now(), root.as_path().to_str().unwrap()).unwrap();

        let loc_b = Path::new(root.as_path()).join(".outpack/location/be7a7bcb");
        let entries_b = read_location(loc_b.clone()).unwrap();
        let res = entries_b.iter().find(|e| e.packet == entry_a.packet).unwrap();
        assert_eq!(res.time, time_as_num(now));

        mark_packet_known(&entry_a.packet, "be7a7bcb", &entry_a.hash,
                          SystemTime::now() + Duration::from_secs(120), root.as_path().to_str().unwrap()).unwrap();

        let entries_b = read_location(loc_b).unwrap();
        let res = entries_b.iter().find(|e| e.packet == entry_a.packet).unwrap();
        // time known should still be the time it was first added at
        assert_eq!(res.time, time_as_num(now));
    }
}
