use serde::{Deserialize, Serialize};
use lazy_static::lazy_static;
use regex::Regex;
use sha1::Digest;
use std::fmt;
use std::fmt::LowerHex;
use std::path::Path;

#[derive(Clone, Copy, Debug, PartialEq)]
#[derive(Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum HashAlgorithm {
    Md5,
    Sha1,
    Sha256,
    Sha384,
    Sha512,
}

#[derive(Debug, PartialEq)]
pub struct Hash {
    pub algorithm: HashAlgorithm,
    pub value: String,
}

#[derive(Debug, PartialEq)]
pub struct ParseHashAlgorithmErr;

#[derive(Debug, PartialEq)]
pub enum ParseHashErr {
    InvalidFormat,
    InvalidAlgorithm,
}

#[derive(Debug, PartialEq)]
pub enum HashValidateErr {
    HashesDontMatch,
    InvalidExpectedHash,
    FileReadFailed,
}

// This is still a bit of a mess, not least because we generally end
// up casting through to io::Error and then through to a server error
// type. I've largely left this as-is for now, but will do one quick
// tidy up before submission I hope. Later we might explore 'anyhow'
// for fairly nice error handling.
impl From<ParseHashAlgorithmErr> for ParseHashErr {
    fn from(_: ParseHashAlgorithmErr) -> Self {
        ParseHashErr::InvalidAlgorithm
    }
}

impl From<std::io::Error> for HashValidateErr {
    fn from(_: std::io::Error) -> Self {
        HashValidateErr::FileReadFailed
    }
}

impl From<ParseHashErr> for HashValidateErr {
    fn from(_: ParseHashErr) -> Self {
        HashValidateErr::InvalidExpectedHash
    }
}

impl fmt::Display for HashAlgorithm {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self {
            Self::Md5 => "md5",
            Self::Sha1 => "sha1",
            Self::Sha256 => "sha256",
            Self::Sha384 => "sha384",
            Self::Sha512 => "sha512",
        };
        write!(f, "{}", s)
    }
}

impl fmt::Display for Hash {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}:{}", self.algorithm, self.value)
    }
}

impl std::str::FromStr for HashAlgorithm {
    type Err = ParseHashAlgorithmErr;

    fn from_str(s: &str) -> Result<HashAlgorithm, Self::Err> {
        match s {
            "md5" => Ok(HashAlgorithm::Md5),
            "sha1" => Ok(HashAlgorithm::Sha1),
            "sha256" => Ok(HashAlgorithm::Sha256),
            "sha384" => Ok(HashAlgorithm::Sha384),
            "sha512" => Ok(HashAlgorithm::Sha512),
            _ => Err(ParseHashAlgorithmErr),
        }
    }
}

impl std::str::FromStr for Hash {
    type Err = ParseHashErr;

    fn from_str(s: &str) -> Result<Hash, Self::Err> {
        lazy_static! {
            static ref HASH_RE: Regex =
                Regex::new(r"^\s*(?<algorithm>[[:alnum:]]+):(?<value>[[:xdigit:]]+)\s*$")
                    .expect("Valid regex");
        }
        let caps = HASH_RE
            .captures(s)
            .ok_or(ParseHashErr::InvalidFormat)?;
        let algorithm = &caps["algorithm"];
        let algorithm: HashAlgorithm = algorithm.parse()?;
        let value = String::from(&caps["value"]);
        Ok(Hash { algorithm, value })
    }
}

fn hex_string<T: LowerHex>(digest: T) -> String {
    format!("{:x}", digest)
}

pub fn hash_data(data: &[u8], algorithm: HashAlgorithm) -> Hash {
    let value: String = match algorithm {
        HashAlgorithm::Md5 => hex_string(md5::compute(data)),
        HashAlgorithm::Sha1 => hex_string(sha1::Sha1::new().chain_update(data).finalize()),
        HashAlgorithm::Sha256 => hex_string(sha2::Sha256::new().chain_update(data).finalize()),
        HashAlgorithm::Sha384 => hex_string(sha2::Sha384::new().chain_update(data).finalize()),
        HashAlgorithm::Sha512 => hex_string(sha2::Sha512::new().chain_update(data).finalize()),
    };
    Hash { algorithm, value }
}

pub fn hash_file(path: &Path, algorithm: HashAlgorithm) -> Result<Hash, std::io::Error> {
    let bytes = std::fs::read(path)?;
    Ok(hash_data(&bytes, algorithm))
}

pub fn validate_hash(found: &Hash, expected: &Hash) -> Result<(), HashValidateErr> {
    if *found != *expected {
        Err(HashValidateErr::HashesDontMatch)
    } else {
        Ok(())
    }
}

pub fn validate_hash_data(data: &[u8], expected: &str) -> Result<(), HashValidateErr> {
    let expected: Hash = expected.parse()?;
    validate_hash(&hash_data(data, expected.algorithm), &expected)
}

// This is not yet a streaming hash, which can be done!
pub fn validate_hash_file(path: &Path, expected: &str) -> Result<(), HashValidateErr> {
    let expected: Hash = expected.parse()?;
    validate_hash(&hash_file(path, expected.algorithm)?, &expected)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile;

    #[test]
    fn can_deparse_hash_algorithm() {
        assert_eq!(format!("{}", HashAlgorithm::Md5), "md5");
        assert_eq!(format!("{}", HashAlgorithm::Sha1), "sha1");
        assert_eq!(format!("{}", HashAlgorithm::Sha256), "sha256");
        assert_eq!(format!("{}", HashAlgorithm::Sha384), "sha384");
        assert_eq!(format!("{}", HashAlgorithm::Sha512), "sha512");
    }

    #[test]
    fn can_parse_hash_algorithm() {
        assert_eq!("md5".parse(), Ok(HashAlgorithm::Md5));
        assert_eq!("sha1".parse(), Ok(HashAlgorithm::Sha1));
        assert_eq!("sha256".parse(), Ok(HashAlgorithm::Sha256));
        assert_eq!("sha384".parse(), Ok(HashAlgorithm::Sha384));
        assert_eq!("sha512".parse(), Ok(HashAlgorithm::Sha512));
        assert_eq!(
            "sha3-256".parse::<HashAlgorithm>(),
            Err(ParseHashAlgorithmErr)
        );
    }

    #[test]
    fn can_deparse_hash() {
        let h = Hash {
            algorithm: HashAlgorithm::Md5,
            value: String::from("123"),
        };
        assert_eq!(format!("{}", h), "md5:123");
    }

    #[test]
    fn can_parse_hash() {
        assert_eq!(
            "md5:1234".parse(),
            Ok(Hash {
                algorithm: HashAlgorithm::Md5,
                value: String::from("1234")
            })
        );
        assert_eq!(
            " sha256:abcde".parse(),
            Ok(Hash {
                algorithm: HashAlgorithm::Sha256,
                value: String::from("abcde")
            })
        );
        assert_eq!("md51234".parse::<Hash>(), Err(ParseHashErr::InvalidFormat));
        assert_eq!(
            "sha666:1234".parse::<Hash>(),
            Err(ParseHashErr::InvalidAlgorithm)
        );
    }

    #[test]
    fn can_hash_simple_data() {
        /*
        > orderly2:::hash_data("1234", "md5")
        [1] "md5:81dc9bdb52d04dc20036dbd8313ed055"
        > orderly2:::hash_data("1234", "sha1")
        [1] "sha1:7110eda4d09e062aa5e4a390b0a572ac0d2c0220"
        > orderly2:::hash_data("1234", "sha256")
        [1] "sha256:03ac674216f3e15c761ee1a5e255f067953623c8b388b4459e13f978d7c846f4"
        > orderly2:::hash_data("1234", "sha384")
        [1] "sha384:504f008c8fcf8b2ed5dfcde752fc5464ab8ba064215d9c5b5fc486af3d9ab8c81b14785180d2ad7cee1ab792ad44798c"
        > orderly2:::hash_data("1234", "sha512")
        [1] "sha512:d404559f602eab6fd602ac7680dacbfaadd13630335e951f097af3900e9de176b6db28512f2e000b9d04fba5133e8b1c6e8df59db3a8ab9d60be4b97cc9e81db"
         */
        let data = b"1234";
        let expect_md5 = "md5:81dc9bdb52d04dc20036dbd8313ed055";
        let expect_sha1 = "sha1:7110eda4d09e062aa5e4a390b0a572ac0d2c0220";
        let expect_sha256 =
            "sha256:03ac674216f3e15c761ee1a5e255f067953623c8b388b4459e13f978d7c846f4";
        let expect_sha384 =
            "sha384:504f008c8fcf8b2ed5dfcde752fc5464ab8ba064215d9c5b5fc486af3d9ab8c81b14785180d2ad7cee1ab792ad44798c";
        let expect_sha512 =
            "sha512:d404559f602eab6fd602ac7680dacbfaadd13630335e951f097af3900e9de176b6db28512f2e000b9d04fba5133e8b1c6e8df59db3a8ab9d60be4b97cc9e81db";
        assert_eq!(
            hash_data(data, HashAlgorithm::Md5),
            expect_md5.parse::<Hash>().unwrap()
        );
        assert_eq!(
            hash_data(data, HashAlgorithm::Sha1),
            expect_sha1.parse::<Hash>().unwrap()
        );
        assert_eq!(
            hash_data(data, HashAlgorithm::Sha256),
            expect_sha256.parse::<Hash>().unwrap()
        );
        assert_eq!(
            hash_data(data, HashAlgorithm::Sha384),
            expect_sha384.parse::<Hash>().unwrap()
        );
        assert_eq!(
            hash_data(data, HashAlgorithm::Sha512),
            expect_sha512.parse::<Hash>().unwrap()
        );
    }

    #[test]
    fn can_validate_hash() {
        let expect_md5 = "md5:81dc9bdb52d04dc20036dbd8313ed055";
        assert_eq!(validate_hash_data(b"1234", &expect_md5), Ok(()));
        assert_eq!(
            validate_hash_data(b"12345", expect_md5),
            Err(HashValidateErr::HashesDontMatch)
        );
    }

    #[test]
    fn can_hash_file() {
        use std::io::Write;
        use tempfile::NamedTempFile;
        let mut file = NamedTempFile::new().unwrap();
        write!(file, "Hello World!").unwrap();
        file.flush().unwrap();
        let expect: Hash = "sha1:2ef7bde608ce5404e97d5f042f95f89f1c232871"
            .parse()
            .unwrap();
        assert_eq!(hash_file(file.path(), HashAlgorithm::Sha1).unwrap(), expect)
    }

    #[test]
    fn can_validate_file() {
        use std::io::Write;
        use tempfile::NamedTempFile;
        let mut file = NamedTempFile::new().unwrap();
        write!(file, "Hello World!").unwrap();
        file.flush().unwrap();
        let expected = "sha1:2ef7bde608ce5404e97d5f042f95f89f1c232871";
        let unexpected = "sha1:2ef7bde608ce5404e97d5f042f95f89f1c232872";
        assert_eq!(validate_hash_file(file.path(), expected), Ok(()));
        assert_eq!(
            validate_hash_file(file.path(), unexpected),
            Err(HashValidateErr::HashesDontMatch)
        );
        assert_eq!(
            validate_hash_file(file.path().join("more").as_path(), expected),
            Err(HashValidateErr::FileReadFailed)
        );
    }
}
