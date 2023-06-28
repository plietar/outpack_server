use sha1::{Sha1};
use sha2::{Sha256, Sha512, Sha384, Digest};
use std::io;
use std::io::{Error, ErrorKind};
use regex::Regex;
use crate::config;

use crate::config::HashAlgorithm;

const HASH_REG: &str = "^([[:alnum:]]+):([[:xdigit:]]+)$";

#[derive(Debug)]
pub struct ParsedHash {
    pub algorithm: String,
    pub value: String,
}

fn invalid_hash(hash: &str) -> io::Error {
    io::Error::new(ErrorKind::InvalidInput, format!("invalid hash '{}'", hash))
}

pub fn hash_parse(hash: &str) -> io::Result<ParsedHash> {
    let hash = hash.trim();
    let hash_reg = Regex::new(HASH_REG).expect("Valid regex");
    let caps = hash_reg.captures(hash)
        .ok_or_else(|| invalid_hash(hash))?;
    let algorithm = caps.get(1).map(|m| String::from(m.as_str()))
        .ok_or_else(|| invalid_hash(hash))?;
    let value = caps.get(2).map(|m| String::from(m.as_str()))
        .ok_or_else(|| invalid_hash(hash))?;

    Ok(ParsedHash { algorithm, value })
}

pub fn hash_data(data: &str, algorithm: HashAlgorithm) -> String {
    match algorithm {
        HashAlgorithm::md5 => format!("md5:{:x}", md5::compute(data)),
        HashAlgorithm::sha1 => format!("sha1:{:x}", Sha1::new()
            .chain_update(data)
            .finalize()),
        HashAlgorithm::sha256 => format!("sha256:{:x}", Sha256::new()
            .chain_update(data)
            .finalize()),
        HashAlgorithm::sha384 => format!("sha384:{:x}", Sha384::new()
            .chain_update(data)
            .finalize()),
        HashAlgorithm::sha512 => format!("sha512:{:x}", Sha512::new()
            .chain_update(data)
            .finalize()),
    }
}

pub fn validate_hash(root: &str, hash: &str, content: &str) -> Result<(), Error> {
    let parsed = hash_parse(hash)?;
    let alg = config::read_config(root)?.core.hash_algorithm;
    if parsed.algorithm != alg.to_string() {
        return Err(Error::new(ErrorKind::InvalidInput,
                              format!("Hash algorithm {} does not match config. Expected {}",
                                      parsed.algorithm, alg)));
    }
    let valid_hash = hash_data(content, alg);
    if hash != valid_hash {
        return Err(Error::new(ErrorKind::InvalidInput,
                              format!("Hash {} does not match file contents. Expected {}",
                                      hash, valid_hash)));
    }
    Ok(())
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
    fn can_hash_data() {
        let data = "1234";
        let expected = format!("{:x}", md5::compute(data));
        let res = hash_parse(&hash_data(data, HashAlgorithm::md5)).unwrap();
        assert_eq!(res.algorithm, "md5");
        assert_eq!(res.value, expected);
    }

    #[test]
    fn error_on_invalid_hash() {
        let hash = "123456";
        let res = hash_parse(&hash);
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "invalid hash '123456'");
    }

    #[test]
    fn validates_hash() {
        let hash = "badhash";
        let res = validate_hash("tests/example", hash, "1234");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "invalid hash 'badhash'");
    }

    #[test]
    fn validates_hash_algorithm() {
        let hash = "md5:e9aa9f2212aba6fba4464212800a2927afa02eda688cf13131652da307e3d7c1";
        let res = validate_hash("tests/example", hash, "1234");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "Hash algorithm md5 does not match config. Expected sha256");
    }

    #[test]
    fn validates_hash_contents() {
        let hash = "sha256:e9aa9f2212aba6fba4464212800a2927afa02eda688cf13131652da307e3d7c1";
        let res = validate_hash("tests/example", hash, "1234");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(),
                   "Hash sha256:e9aa9f2212aba6fba4464212800a2927afa02eda688cf13131652da307e3d7c1 does not match file contents. Expected sha256:03ac674216f3e15c761ee1a5e255f067953623c8b388b4459e13f978d7c846f4");
    }
}
