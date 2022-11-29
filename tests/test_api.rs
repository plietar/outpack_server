use std::fs;
use std::io::prelude::*;
use std::path::Path;
use std::sync::Arc;
use rocket::local::blocking::Client;
use rocket::http::{ContentType, Status};
use jsonschema::{Draft, JSONSchema, SchemaResolverError};
use serde_json::Value;
use url::Url;

// rust doesn't have post-test hooks so this is to allow creating temporary
// files that get removed when they go out of scope
struct LocalTempFile {
    file_name: String
}

impl LocalTempFile {
    fn new(file_name: &str, body: &[u8]) -> LocalTempFile {
        let mut file = fs::File::create(file_name).expect("File created");
        file.write_all(body).expect("File written");
        LocalTempFile { file_name: String::from(file_name) }
    }
}

impl Drop for LocalTempFile {
    fn drop(&mut self) {
        fs::remove_file(&self.file_name).expect("File removed");
    }
}


#[test]
fn can_get_index() {
    let rocket = outpack_server::api(String::from("tests/example"));
    let client = Client::tracked(rocket).expect("valid rocket instance");
    let response = client.get("/").dispatch();

    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response.content_type(), Some(ContentType::JSON));

    let body = serde_json::from_str(&response.into_string().unwrap()).unwrap();
    validate_success("root.json", &body);
}

#[test]
fn error_if_cant_get_index() {
    let rocket = outpack_server::api(String::from("badlocation"));
    let client = Client::tracked(rocket).expect("valid rocket instance");
    let response = client.get("/").dispatch();

    assert_eq!(response.status(), Status::InternalServerError);
    assert_eq!(response.content_type(), Some(ContentType::JSON));

    let body = serde_json::from_str(&response.into_string().unwrap()).unwrap();
    validate_error(&body, Some("No such file or directory"));
}

#[test]
fn can_get_metadata() {
    let rocket = outpack_server::api(String::from("tests/example"));
    let client = Client::tracked(rocket).expect("valid rocket instance");
    let response = client.get("/metadata/list").dispatch();

    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response.content_type(), Some(ContentType::JSON));

    let body: Value = serde_json::from_str(&response.into_string().unwrap()).unwrap();
    validate_success("list.json", &body);

    let entries = body.get("data").unwrap().as_array().unwrap();
    assert_eq!(entries.len(), 3);

    assert_eq!(entries[0].get("packet").unwrap().as_str().unwrap(), "20170818-164847-7574883b");
    assert_eq!(entries[0].get("time").unwrap().as_f64().unwrap(), 1662480556.1778);
    assert_eq!(entries[0].get("hash").unwrap().as_str().unwrap(),
               "sha256:af3c863f96898c6c88cee4daa1a6d6cfb756025e70059f5ea4dbe4d9cc5e0e36");

    assert_eq!(entries[1].get("packet").unwrap().as_str().unwrap(), "20170818-164043-7cdcde4b");
    assert_eq!(entries[2].get("packet").unwrap().as_str().unwrap(), "20170818-164830-33e0ab01");
}

#[test]
fn handles_metadata_errors() {
    let rocket = outpack_server::api(String::from("tests/example"));
    let client = Client::tracked(rocket).expect("valid rocket instance");
    let file_name = "tests/example/.outpack/location/ae7a7bcb/20180818-164043-7cdcde4b";
    let _ = LocalTempFile::new(file_name , b"{}");
    let response = client.get("/metadata/list").dispatch();
    assert_eq!(response.status(), Status::InternalServerError);
    assert_eq!(response.content_type(), Some(ContentType::JSON));

    let body = serde_json::from_str(&response.into_string().unwrap()).unwrap();
    validate_error(&body, Some("missing field `packet`"));
}

#[test]
fn catches_404() {
    let rocket = outpack_server::api(String::from("tests/example"));
    let client = Client::tracked(rocket).expect("valid rocket instance");
    let response = client.get("/badurl").dispatch();

    assert_eq!(response.status(), Status::NotFound);
    assert_eq!(response.content_type(), Some(ContentType::JSON));

    let body = serde_json::from_str(&response.into_string().unwrap()).unwrap();
    validate_error(&body, Some("This route does not exist"));
}

fn validate_success(schema_name: &str, instance: &Value) {
    let compiled_schema = get_schema("response-success.json");
    assert_valid(instance, &compiled_schema);
    let status = instance.get("status")
        .expect("Status property present");
    assert_eq!(status, "success");

    let data = instance.get("data")
        .expect("Data property present");
    let compiled_schema = get_schema(schema_name);
    assert_valid(data, &compiled_schema);
}

fn validate_error(instance: &Value, message: Option<&str>) {
    let compiled_schema = get_schema("response-failure.json");
    assert_valid(instance, &compiled_schema);
    let status = instance.get("status")
        .expect("Status property present");
    assert_eq!(status, "failure");

    if message.is_some() {
        let err = instance.get("errors")
            .expect("Status property present")
            .as_array().unwrap().get(0)
            .expect("First error")
            .get("detail")
            .expect("Error detail")
            .to_string();

        assert!(err.contains(message.unwrap()));

    }
}

fn assert_valid(instance: &Value, compiled: &JSONSchema) {
    let result = compiled.validate(&instance);
    if let Err(errors) = result {
        for error in errors {
            println!("Validation error: {}", error);
            println!("Instance path: {}", error.instance_path);
        }
    }
    assert!(compiled.is_valid(&instance));
}

fn get_schema(schema_name: &str) -> JSONSchema {
    let schema_path = Path::new("schema")
        .join(schema_name);
    let schema_as_string = fs::read_to_string(schema_path)
        .expect("Schema file");

    let json_schema = serde_json::from_str(&schema_as_string)
        .expect("Schema is valid json");

    JSONSchema::options()
        .with_draft(Draft::Draft7)
        .with_resolver(LocalSchemaResolver {})
        .compile(&json_schema)
        .expect("A valid schema")
}

struct LocalSchemaResolver;

impl jsonschema::SchemaResolver for LocalSchemaResolver {
    fn resolve(&self, _root_schema: &Value, _url: &Url, original_reference: &str) -> Result<Arc<Value>, SchemaResolverError> {
        let schema_path = Path::new("schema")
            .join(original_reference);
        let schema_as_string = fs::read_to_string(schema_path)
            .expect("Schema file");
        let json_schema = serde_json::from_str(&schema_as_string)
            .expect("Schema is valid json");
        return Ok(Arc::new(json_schema));
    }
}
