use std::ffi::{OsString};
use regex::Regex;

pub const ID_REG: &str = "^([0-9]{8}-[0-9]{6}-[[:xdigit:]]{8})$";

pub fn is_packet(name: &OsString, reg: &Regex) -> bool {
    let o = name.to_str();
    o.map_or(false, |s| reg.is_match(s))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_detect_packet_id() {
        let reg = Regex::new(ID_REG).unwrap();
        assert_eq!(is_packet(&OsString::from("1234"), &reg), false);
        assert_eq!(is_packet(&OsString::from("20170818-164830-33e0ab01"), &reg), true);
        assert_eq!(is_packet(&OsString::from("20180818-164847-54699abf"), &reg), true)
    }
}
