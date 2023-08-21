use std::fs;
use std::io::Error;
use std::path::Path; // still using io errors here due to config

use crate::config;

pub fn outpack_init(
    path: &str,
    path_archive: Option<String>,
    use_file_store: bool,
    require_complete_tree: bool,
) -> Result<(), Error> {
    let path_outpack = Path::new(path).join(".outpack");
    let cfg = config::Config::new(path_archive, use_file_store, require_complete_tree)?;

    if path_outpack.exists() {
        let prev = config::read_config(path)?;
        if cfg.core != prev.core {
            return Err(Error::new(
                std::io::ErrorKind::InvalidInput,
                "Trying to change config on reinitialisation",
            ));
        }
    } else {
        fs::create_dir_all(&path_outpack)?;
        config::write_config(&cfg, path)?;
        fs::create_dir_all(&path_outpack.join("location").join("local"))?;
        fs::create_dir_all(&path_outpack.join("metadata"))?;
        if use_file_store {
            fs::create_dir_all(path_outpack.join("files"))?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile;

    #[test]
    fn can_create_empty_config() {
        let tmp = tempfile::TempDir::new().unwrap();
        let path = tmp.path();
        let path_str = path.to_str().unwrap();
        let res = outpack_init(path_str, None, true, true);
        assert!(res.is_ok());
        assert_eq!(
            config::read_config(path_str).unwrap(),
            config::Config::new(None, true, true).unwrap()
        );
    }

    #[test]
    fn can_reinit_an_existing_repo() {
        let tmp = tempfile::TempDir::new().unwrap();
        let path = tmp.path();
        let path_str = path.to_str().unwrap();
        let res = outpack_init(path_str, Some(String::from("archive")), false, false);
        assert!(res.is_ok());
        assert_eq!(
            config::read_config(path_str).unwrap(),
            config::Config::new(Some(String::from("archive")), false, false).unwrap()
        );

        let res = outpack_init(path_str, Some(String::from("archive")), false, false);
        assert!(res.is_ok());
        assert_eq!(
            config::read_config(path_str).unwrap(),
            config::Config::new(Some(String::from("archive")), false, false).unwrap()
        );
    }

    #[test]
    fn error_if_config_has_changed() {
        let tmp = tempfile::TempDir::new().unwrap();
        let path = tmp.path();
        let path_str = path.to_str().unwrap();
        outpack_init(path_str, Some(String::from("archive")), false, false).unwrap();
        let res = outpack_init(path_str, None, true, true);
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().to_string(),
            "Trying to change config on reinitialisation"
        )
    }
}
