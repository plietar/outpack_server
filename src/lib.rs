use std::io;
use rocket::serde::{Deserialize, Serialize};
use rocket::serde::json::{Json};
use rocket::http::{ContentType};
use rocket::{Build, Rocket};
use rocket::State;

#[macro_use]
extern crate rocket;

mod config;

#[allow(clippy::result_large_err)]
#[get("/")]
fn index(root: &State<String>) -> Result<SuccessResponder<config::Config>, ErrorResponder> {
    Ok(SuccessResponder::from(config::read_config(root)?))
}

pub fn api(root: String) -> Rocket<Build> {
    rocket::build()
        .manage(root)
        .mount("/", routes![index])
}

#[derive(Responder)]
#[response(status = 500, content_type = "json")]
struct ErrorResponder {
    inner: Json<ApiError>,
    header: ContentType,
}

#[derive(Responder)]
#[response(status = 200, content_type = "json")]
struct SuccessResponder<T> {
    inner: Json<Response<T>>,
    header: ContentType,
}

#[derive(Serialize, Deserialize, Debug)]
struct ApiError {
    message: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Response<T> {
    status: String,
    data: T,
    errors: Option<ApiError>
}

impl From<io::Error> for ErrorResponder {
    fn from(e: io::Error) -> Self {
        ErrorResponder { inner: Json(ApiError { message: e.to_string() }), header: ContentType::JSON }
    }
}

impl<T> From<T> for SuccessResponder<T> {
    fn from(obj: T) -> Self {
        SuccessResponder {
            inner: Json(Response {
                status: String::from("success"),
                data: obj,
                errors: None,
            }),
            header: ContentType::JSON,
        }
    }
}
