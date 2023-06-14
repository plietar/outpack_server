use std::ffi::{OsString};
use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    static ref ID_REG: Regex = Regex::new(r"^([0-9]{8}-[0-9]{6}-[[:xdigit:]]{8})$").unwrap();
}

pub fn is_packet(name: &OsString) -> bool {
    let o = name.to_str();
    o.map_or(false, is_packet_str)
}

pub fn is_packet_str(name: &str) -> bool {
    ID_REG.is_match(name)
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_detect_packet_id() {
        assert_eq!(is_packet(&OsString::from("1234")), false);
        assert_eq!(is_packet(&OsString::from("20170818-164830-33e0ab01")), true);
        assert_eq!(is_packet(&OsString::from("20180818-164847-54699abf")), true)
    }
}
