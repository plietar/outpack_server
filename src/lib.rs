use rocket::serde::json::{Json};
use rocket::{Build, catch, catchers, Request, Rocket, routes};
use rocket::State;

mod config;
mod responses;
mod location;

use responses::{FailResponse, OutpackError, OutpackSuccess};
pub use self::location::LocationEntry;

type OutpackResult<T> = Result<OutpackSuccess<T>, OutpackError>;

#[catch(500)]
fn internal_error(_req: &Request) -> Json<FailResponse> {
    Json(FailResponse::from(OutpackError {
        error: String::from("UNKNOWN_ERROR"),
        detail: String::from("Something went wrong")
    }))
}

#[catch(404)]
fn not_found(_req: &Request) -> Json<FailResponse> {
    Json(FailResponse::from(OutpackError {
        error: String::from("NOT_FOUND"),
        detail: String::from("This route does not exist")
    }))
}

#[rocket::get("/")]
fn index(root: &State<String>) -> OutpackResult<config::Config> {
    config::read_config(root)
        .map_err(|e| OutpackError::new(e))
        .map(|r| OutpackSuccess::from(r))
}

#[allow(clippy::result_large_err)]
#[rocket::get("/metadata/list")]
fn list(root: &State<String>) -> OutpackResult<Vec<location::LocationEntry>> {
    location::read_locations(root)
        .map_err(|e| OutpackError::new(e))
        .map(|r| OutpackSuccess::from(r))
}

pub fn api(root: String) -> Rocket<Build> {
    rocket::build()
        .manage(root)
        .register("/", catchers![internal_error, not_found])
        .mount("/", routes![index, list])
}
