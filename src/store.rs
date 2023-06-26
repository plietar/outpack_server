use std::{fs, io};
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use rocket::fs::TempFile;
use crate::config;

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

pub fn file_exists(root: &str, hash: &str) -> io::Result<bool> {
    let path = file_path(root, hash)?;
    Ok(fs::metadata(path).is_ok())
}

pub fn get_missing_files(root: &str, wanted: &[String]) -> io::Result<Vec<String>> {
    wanted.iter()
        .filter_map(|h| match file_exists(root, h) {
            Ok(false) => Some(Ok(h.clone())),
            Ok(true) => None,
            Err(e) => Some(Err(e)),
        })
        .collect()
}

pub async fn put_file(root: &str, mut file: TempFile<'_>, hash: &str) -> io::Result<String> {
    let temp_path = std::env::temp_dir().join(hash);
    file.persist_to(&temp_path).await?;

    let alg = config::read_config(root)?.core.hash_algorithm;
    let content = fs::read_to_string(&temp_path)?;

    if hash != hash::hash_data(content, alg) {
        return Err(io::Error::new(ErrorKind::InvalidInput,
                                  "Hash does not match file contents"));
    }

    let path = file_path(root, hash)?;
    let pathname = (path).to_str()
        .map(String::from)
        .unwrap();

    if !file_exists(root, hash)? {
        fs::create_dir_all(path.parent().unwrap())?;
        fs::rename(temp_path, path)
            .map(|_| pathname)
    } else {
        Ok(pathname)
    }
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;
    use crate::config::HashAlgorithm;
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

    #[rocket::async_test]
    async fn put_file_is_idempotent() {
        let root = tempdir().unwrap();
        let data = "Testing 123.";
        let temp_file = TempFile::Buffered {
            content: data
        };
        let hash = hash::hash_data(String::from(data), HashAlgorithm::sha256);
        let root_path = root.path();
        let outpack_path = root_path.join(".outpack");
        fs::create_dir(&outpack_path).unwrap();
        fs::copy("tests/example/.outpack/config.json", outpack_path.join("config.json")).unwrap();

        let root_str = root_path.to_str().unwrap();
        let res = put_file(root_str, temp_file, &hash);
        let expected = file_path(root_str, &hash);
        assert_eq!(res.await.unwrap(), expected.unwrap().to_str().unwrap());

        let temp_file = TempFile::Buffered {
            content: data
        };
        let res = put_file(root_str, temp_file, &hash);
        let expected = file_path(root_str, &hash);
        assert_eq!(res.await.unwrap(), expected.unwrap().to_str().unwrap());
    }

    #[rocket::async_test]
    async fn put_file_validates_hash() {
        let root = tempdir().unwrap();
        let data = "Testing 123.";
        let temp_file = TempFile::Buffered {
            content: data
        };
        let root_path = root.path();
        let outpack_path = root_path.join(".outpack");
        fs::create_dir(&outpack_path).unwrap();
        fs::copy("tests/example/.outpack/config.json", outpack_path.join("config.json")).unwrap();

        let root_str = root_path.to_str().unwrap();
        let res = put_file(root_str, temp_file, "badhash");
        assert_eq!(res.await.unwrap_err().to_string(), "Hash does not match file contents");
    }
}
