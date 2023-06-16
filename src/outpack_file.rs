use std::io;
use std::io::ErrorKind;
use rocket::tokio::fs::File;
use std::path::{Path};
use rocket::http::ContentType;

use rocket::response::{Responder, Result};
use rocket::Request;

pub struct OutpackFile {
    hash: String,
    file: File,
    size: u64,
}

impl OutpackFile {
    pub async fn open<P: AsRef<Path>>(hash: String, path: P) -> io::Result<OutpackFile> {
        let file = File::open(path.as_ref()).await.map_err(|e| {
            match e.kind() {
                ErrorKind::NotFound => io::Error::new(ErrorKind::NotFound,
                                                      format!("hash '{}' not found", hash)),
                _ => e
            }
        })?;
        let size = file.metadata().await?.len();
        Ok(OutpackFile { hash, file, size })
    }
}

impl<'r> Responder<'r, 'static> for OutpackFile {
    fn respond_to(self, request: &'r Request<'_>) -> Result<'static> {
        use rocket::http::hyper::header::*;

        let content_type = ContentType::Binary.to_string();
        let content_disposition = format!("attachment; filename=\"{}\"", self.hash);

        let mut response = self.file.respond_to(request)?;
        response.set_raw_header(CONTENT_TYPE.as_str(), content_type);
        response.set_raw_header(CONTENT_DISPOSITION.as_str(), content_disposition);
        response.set_raw_header(CONTENT_LENGTH.as_str(), self.size.to_string());

        Ok(response)
    }
}
