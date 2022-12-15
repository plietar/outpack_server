use std::io;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use regex::Regex;

const HASH_REG: &str = "^([[:alnum:]]+):([[:xdigit:]]+)$";

#[derive(Debug)]
struct ParsedHash {
    algorithm: String,
    value: String,
}

fn invalid_hash(hash: &str) -> io::Error {
    io::Error::new(ErrorKind::Other, format!("invalid hash '{}'", hash))
}

fn hash_parse(hash: &str) -> io::Result<ParsedHash> {
    let hash_reg = Regex::new(HASH_REG).expect("Valid regex");
    let caps = hash_reg.captures(hash)
        .ok_or(invalid_hash(hash))?;
    let algorithm = caps.get(1).map(|m| String::from(m.as_str()))
        .ok_or(invalid_hash(hash))?;
    let value = caps.get(2).map(|m| String::from(m.as_str()))
        .ok_or(invalid_hash(hash))?;
    Ok(ParsedHash { algorithm, value })
}

pub fn file_path(root: &str, hash: &str) -> io::Result<PathBuf> {
    let parsed = hash_parse(hash)?;
    Ok(Path::new(root)
        .join(".outpack")
        .join("files")
        .join(parsed.algorithm)
        .join(&parsed.value[..2])
        .join(&parsed.value[2..]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_parse_hash() {
        let hash = "sha256:e9aa9f2212aba6fba4464212800a2927afa02eda688cf13131652da307e3d7c1";
        let res = hash_parse(&hash).unwrap();
        assert_eq!(res.algorithm, "sha256");
        assert_eq!(res.value, "e9aa9f2212aba6fba4464212800a2927afa02eda688cf13131652da307e3d7c1");
    }

    #[test]
    fn error_on_invalid_hash() {
        let hash = "123456";
        let res = hash_parse(&hash);
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "invalid hash '123456'");
    }

    #[test]
    fn can_get_path() {
        let hash = "sha256:e9aa9f2212ab";
        let res = file_path("root", hash).unwrap();
        assert_eq!(res.to_str().unwrap(), "root/.outpack/files/sha256/e9/aa9f2212ab");
    }

    #[test]
    fn path_propagates_error_on_invalid_hash() {
        let hash = "sha256";
        let res = file_path("root", hash);
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "invalid hash 'sha256'");
    }
}
