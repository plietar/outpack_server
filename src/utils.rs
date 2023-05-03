use std::io::{Error, ErrorKind};
use std::ffi::{OsString};
use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    static ref ID_REG: Regex = Regex::new(r"^([0-9]{8}-[0-9]{6}-[[:xdigit:]]{8})$").unwrap();
    static ref DATETIME_REG: Regex = Regex::new(r"^([0-9]{8}(-[0-9]{6})?)$").unwrap();
}

pub fn is_packet(name: &OsString) -> bool {
    let o = name.to_str();
    o.map_or(false, |s| ID_REG.is_match(s))
}

pub fn validate_datetime(input: &Option<String>) -> Result<bool, Error> {
    match input {
        None => Ok(true),
        Some(str) => if DATETIME_REG.is_match(&str) {
            Ok(true)
        } else {
            Err(Error::new(ErrorKind::Other,
                           format!("Not an outpack datetime. Format should be YYMMDD-HHMMSS.")))
        }
    }
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

    #[test]
    fn can_detect_datetime_format() {
        assert_eq!(validate_datetime(&None).unwrap(), true);
        assert_eq!(validate_datetime(&Some(String::from("2018081"))).is_err(), true);
        assert_eq!(validate_datetime(&Some(String::from("20170818-16483"))).is_err(), true);
        assert_eq!(validate_datetime(&Some(String::from("20170818-16483a"))).is_err(), true);
        assert_eq!(validate_datetime(&Some(String::from("20170818-164830"))).unwrap(), true);
        assert_eq!(validate_datetime(&Some(String::from("20180818"))).unwrap(), true)
    }
}
