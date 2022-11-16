use std::io;
use rocket::serde::{Deserialize, Serialize};
use rocket::serde::json::{Json};
use rocket::http::{ContentType};
use rocket::{Build, Rocket};
use rocket::State;

mod config;

#[allow(clippy::result_large_err)]
#[rocket::get("/")]
fn index(root: &State<String>) -> Result<SuccessResponder<config::Config>, ErrorResponder> {
    Ok(SuccessResponder::from(config::read_config(root)?))
}

pub fn api(root: String) -> Rocket<Build> {
    rocket::build()
        .manage(root)
        .mount("/", rocket::routes![index])
}

#[derive(rocket::Responder)]
#[response(status = 500, content_type = "json")]
struct ErrorResponder {
    inner: Json<FailResponse>,
    header: ContentType
}

#[derive(rocket::Responder)]
#[response(status = 200, content_type = "json")]
struct SuccessResponder<T> {
    inner: Json<SuccessResponse<T>>,
    header: ContentType
}

#[derive(Serialize, Deserialize, Debug)]
struct ApiError {
    error: String,
    detail: String
}

#[derive(Serialize, Deserialize, Debug)]
struct SuccessResponse<T> {
    status: String,
    data: T,
    errors: Option<Vec<ApiError>>
}

#[derive(Serialize, Deserialize, Debug)]
struct FailResponse {
    status: String,
    data: Option<String>,
    errors: Option<Vec<ApiError>>
}

impl From<io::Error> for ErrorResponder {
    fn from(e: io::Error) -> Self {
        ErrorResponder {
            inner: Json(FailResponse{
                status: String::from("failure"),
                data: None,
                errors: Some(Vec::from([ApiError { error: String::from("IOERROR"), detail: e.to_string() }]))
            }),
            header: ContentType::JSON,
        }
    }
}

impl<T> From<T> for SuccessResponder<T> {
    fn from(obj: T) -> Self {
        SuccessResponder {
            inner: Json(SuccessResponse {
                status: String::from("success"),
                data: obj,
                errors: None,
            }),
            header: ContentType::JSON,
        }
    }
}
