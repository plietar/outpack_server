use std::path::{Path, PathBuf};
use regex::Regex;

const HASH_REG: &str = "^([[:alnum:]]+):([[:xdigit:]]+)$";

pub fn hash_parse(hash: &str) -> (&str, &str) {
    let hash_reg = Regex::new(HASH_REG).unwrap();
    let caps = hash_reg.captures(hash).unwrap();
    (caps.get(1).map_or("", |m| m.as_str()), caps.get(2).map_or("", |m| m.as_str()))
}

pub fn file_path(root: &str, hash: &str) -> PathBuf {
    let parsed = hash_parse(hash);
    Path::new(root)
        .join(".outpack")
        .join("files")
        .join(parsed.0)
        .join(&parsed.1[..2])
        .join(&parsed.1[2..])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_parse_hash() {
        let hash = "sha256:e9aa9f2212aba6fba4464212800a2927afa02eda688cf13131652da307e3d7c1";
        let res = hash_parse(&hash);
        assert_eq!(res.0, "sha256");
        assert_eq!(res.1, "e9aa9f2212aba6fba4464212800a2927afa02eda688cf13131652da307e3d7c1");
    }

    #[test]
    fn can_get_path() {
        let hash = "sha256:e9aa9f2212ab";
        let res = file_path("root", hash);
        assert_eq!(res.to_str().unwrap(), "root/.outpack/files/sha256/e9/aa9f2212ab");
    }

}
