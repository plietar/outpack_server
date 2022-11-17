use std::io;
use rocket::serde::json::{Json};
use rocket::{Build, catch, catchers, Request, Rocket};
use rocket::State;

mod config;
mod responses;

use responses::{FailResponse, OutpackError, SuccessResponder};
use crate::responses::SuccessResponse;

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
fn index(root: &State<String>) -> Result<SuccessResponder<config::Config>, io::Error> {
    Ok(SuccessResponder::from(config::read_config(root)?))
}

pub fn api(root: String) -> Rocket<Build> {
    rocket::build()
        .manage(root)
        .register("/", catchers![internal_error, not_found])
        .mount("/", rocket::routes![index])
}
