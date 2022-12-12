use std::{fs, io};
use std::path::{Path};

pub fn get_metadata(root_path: &str, id: &str) -> io::Result<serde_json::Value> {
    let path = Path::new(root_path)
        .join(".outpack")
        .join("metadata")
        .join(id);

    return if !path.exists() {
        Err(io::Error::new(io::ErrorKind::NotFound, format!("packet with id '{}' does not exist", id)))
    } else {
        let file = fs::File::open(&path)?;
        let packet = serde_json::from_reader(file)?;
        Ok(packet)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_get_packet() {
        let _packet = get_metadata("tests/example", "20170818-164847-7574883b")
            .unwrap();
    }

}
