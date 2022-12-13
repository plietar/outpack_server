use std::io::ErrorKind;
use std::io;
use rocket::serde::json::{Json};
use rocket::{Build, catch, catchers, Request, Rocket, routes};
use rocket::fs::{NamedFile};
use rocket::http::{ContentType};
use rocket::State;

mod config;
mod responses;
mod location;
mod metadata;
mod store;

use responses::{FailResponse, OutpackError, OutpackSuccess};

type OutpackResult<T> = Result<OutpackSuccess<T>, OutpackError>;

#[catch(500)]
fn internal_error(_req: &Request) -> Json<FailResponse> {
    Json(FailResponse::from(OutpackError {
        error: String::from("UNKNOWN_ERROR"),
        detail: String::from("Something went wrong"),
        kind: Some(ErrorKind::Other),
    }))
}

#[catch(404)]
fn not_found(_req: &Request) -> Json<FailResponse> {
    Json(FailResponse::from(OutpackError {
        error: String::from("NOT_FOUND"),
        detail: String::from("This route does not exist"),
        kind: Some(ErrorKind::NotFound),
    }))
}

#[rocket::get("/")]
fn index(root: &State<String>) -> OutpackResult<config::Root> {
    config::read_config(root)
        .map(|r| config::Root::new(r.schema_version))
        .map_err(|e| OutpackError::new(e))
        .map(|r| OutpackSuccess::from(r))
}

#[rocket::get("/metadata/list")]
fn list_metadata(root: &State<String>) -> OutpackResult<Vec<location::LocationEntry>> {
    location::read_locations(root)
        .map_err(|e| OutpackError::new(e))
        .map(|r| OutpackSuccess::from(r))
}

#[rocket::get("/metadata/<id>/json")]
fn get_metadata(root: &State<String>, id: String) -> OutpackResult<serde_json::Value> {
    metadata::get_metadata(root, &id)
        .map_err(|e| OutpackError::new(e))
        .map(|r| OutpackSuccess::from(r))
}

#[rocket::get("/metadata/<id>/text")]
fn get_metadata_raw(root: &State<String>, id: String) -> Result<String, OutpackError> {
    metadata::get_metadata_text(root, &id)
        .map_err(|e| OutpackError::new(e))
}

#[rocket::get("/file/<hash>")]
pub async fn get_file(root: &State<String>, hash: String) -> (ContentType, Result<NamedFile, OutpackError>) {
    let path = store::file_path(&root, &hash);
    if !path.exists() {
        (ContentType::JSON, Err(OutpackError::new(io::Error::new(ErrorKind::NotFound,
                                               format!("hash '{}' not found", hash)))))
    } else {
        (ContentType::Binary, Ok(NamedFile::open(path).await.unwrap()))
    }
}

pub fn api(root: String) -> Rocket<Build> {
    rocket::build()
        .manage(root)
        .register("/", catchers![internal_error, not_found])
        .mount("/", routes![index, list_metadata, get_metadata,
            get_metadata_raw, get_file])
}
