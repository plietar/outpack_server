use std::io;
use std::io::ErrorKind;
use rocket::serde::{Deserialize, Serialize};
use rocket::serde::json::{Json, json};
use rocket::http::{ContentType, Status};
use rocket::{Request, Response};
use rocket::response::Responder;

#[derive(Responder)]
#[response(status = 200, content_type = "json")]
pub struct OutpackSuccess<T> {
    inner: Json<SuccessResponse<T>>,
    header: ContentType,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct OutpackError {
    pub error: String,
    pub detail: String,

    #[serde(skip_serializing, skip_deserializing)]
    pub kind: Option<ErrorKind>,
}

impl From<io::Error> for OutpackError {
    fn from(e: io::Error) -> Self {
        OutpackError {
            error: e.kind().to_string(),
            detail: e.to_string(),
            kind: Some(e.kind()),
        }
    }
}

impl<'r> Responder<'r, 'static> for OutpackError {
    fn respond_to(self, req: &'r Request<'_>) -> rocket::response::Result<'static> {
        let kind = self.kind;
        let json = FailResponse::from(self);
        let status = match kind {
            Some(ErrorKind::NotFound) => Status::NotFound,
            Some(ErrorKind::InvalidInput) => Status::BadRequest,
            _ =>  Status::InternalServerError
        };
        Response::build_from(json!(json).respond_to(req).unwrap())
            .status(status)
            .header(ContentType::JSON)
            .ok()
    }
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

impl<T> From<T> for OutpackSuccess<T> {
    fn from(obj: T) -> Self {
        OutpackSuccess {
            inner: Json(SuccessResponse {
                status: String::from("success"),
                data: obj,
                errors: None,
            }),
            header: ContentType::JSON,
        }
    }
}

