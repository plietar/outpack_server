use rocket::serde::{Deserialize, Serialize};
use rocket::serde::json::{Json};
use rocket::http::{ContentType};
use rocket::response::Responder;

#[derive(Responder)]
#[response(status = 200, content_type = "json")]
pub struct SuccessResponder<T> {
    inner: Json<SuccessResponse<T>>,
    header: ContentType,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct OutpackError {
    pub error: String,
    pub detail: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SuccessResponse<T> {
    pub status: String,
    pub data: T,
    pub errors: Option<Vec<OutpackError>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FailResponse {
    pub status: String,
    pub data: Option<String>,
    pub errors: Option<Vec<OutpackError>>,
}

impl From<OutpackError> for FailResponse {
    fn from(e: OutpackError) -> Self {
        FailResponse {
            status: String::from("failure"),
            data: None,
            errors: Some(Vec::from([e])),
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
