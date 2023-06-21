use std::{fs, io};
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use super::hash;

pub fn file_path(root: &str, hash: &str) -> io::Result<PathBuf> {
    let parsed = hash::hash_parse(hash)?;
    Ok(Path::new(root)
        .join(".outpack")
        .join("files")
        .join(parsed.algorithm)
        .join(&parsed.value[..2])
        .join(&parsed.value[2..]))
}

pub fn get_missing_files(root: &str, wanted: &Vec<String>) -> io::Result<Vec<String>> {
    let paths = wanted.iter()
        .map(|h| file_path(root, h)).collect::<io::Result<Vec<PathBuf>>>()?;

    paths.iter().filter(|path| fs::metadata(path).is_ok())
        .map(|p| p.file_name())
        .filter(|f| f.is_some())
        .map(|f| f.unwrap().to_str().ok_or_else(invalid_filename).map(String::from))
        .collect::<io::Result<Vec<String>>>()
}

fn invalid_filename() -> io::Error {
    io::Error::new(ErrorKind::InvalidData, format!("Non-UTF8 character in filename"))
}

#[cfg(test)]
mod tests {
    use super::*;

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
