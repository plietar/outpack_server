use std::io::ErrorKind;
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
pub async fn get_file(root: &State<String>, hash: String) -> (ContentType,
                                                              Result<NamedFile, OutpackError>) {
    let path = store::file_path(&root, &hash);
    let result = NamedFile::open(path).await;
    let content_type = if result.is_err() {
        ContentType::JSON
    } else {
        ContentType::Binary
    };
    (content_type, result
        .map_err(|e| OutpackError::new(e, format!("hash '{}' not found", hash))))
}

pub fn api(root: String) -> Rocket<Build> {
    rocket::build()
        .manage(root)
        .register("/", catchers![internal_error, not_found])
        .mount("/", routes![index, list_metadata, get_metadata,
            get_metadata_raw, get_file])
}
