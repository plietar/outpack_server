use rocket::serde::json::{Json};
use rocket::{Build, catch, catchers, Request, Rocket};
use rocket::State;

mod config;
mod responses;

use responses::{FailResponse, OutpackError, OutpackSuccess};

type OutpackResult = Result<OutpackSuccess<config::Config>, OutpackError>;

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
fn index(root: &State<String>) -> OutpackResult {
    config::read_config(root)
        .map_err(|e| OutpackError::new(e))
        .map(|r| OutpackSuccess::from(r))
}

pub fn api(root: String) -> Rocket<Build> {
    rocket::build()
        .manage(root)
        .register("/", catchers![not_found, internal_error])
        .mount("/", rocket::routes![index])
}
