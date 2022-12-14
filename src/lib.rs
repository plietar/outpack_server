use std::io::ErrorKind;
use rocket::serde::json::{Json};
use rocket::{Build, catch, catchers, Request, Rocket, routes};
use rocket::State;

mod config;
mod responses;
mod location;
mod metadata;
mod store;
mod outpack_file;

use responses::{FailResponse, OutpackError, OutpackSuccess};
use crate::outpack_file::OutpackFile;

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
        .map_err(|e| OutpackError::from(e))
        .map(|r| OutpackSuccess::from(r))
}

#[rocket::get("/metadata/list")]
fn list_metadata(root: &State<String>) -> OutpackResult<Vec<location::LocationEntry>> {
    location::read_locations(root)
        .map_err(|e| OutpackError::from(e))
        .map(|r| OutpackSuccess::from(r))
}

#[rocket::get("/metadata/<id>/json")]
fn get_metadata(root: &State<String>, id: String) -> OutpackResult<serde_json::Value> {
    metadata::get_metadata(root, &id)
        .map_err(|e| OutpackError::from(e))
        .map(|r| OutpackSuccess::from(r))
}

#[rocket::get("/metadata/<id>/text")]
fn get_metadata_raw(root: &State<String>, id: String) -> Result<String, OutpackError> {
    metadata::get_metadata_text(root, &id)
        .map_err(|e| OutpackError::from(e))
}

#[rocket::get("/file/<hash>")]
pub async fn get_file(root: &State<String>, hash: String) -> Result<OutpackFile, OutpackError> {
    let path = store::file_path(&root, &hash);
   OutpackFile::open(hash, path).await
        .map_err(|e| OutpackError::from(e))
}

pub fn api(root: String) -> Rocket<Build> {
    rocket::build()
        .manage(root)
        .register("/", catchers![internal_error, not_found])
        .mount("/", routes![index, list_metadata, get_metadata,
            get_metadata_raw, get_file])
}
