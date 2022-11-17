use std::fs;
use std::path::Path;
use std::sync::Arc;
use rocket::local::blocking::Client;
use rocket::http::{ContentType, Status};
use jsonschema::{Draft, JSONSchema, SchemaResolverError};
use serde_json::Value;
use url::Url;

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
    validate_error(&body);
}

#[test]
fn catches_404() {
    let rocket = outpack_server::api(String::from("tests/example"));
    let client = Client::tracked(rocket).expect("valid rocket instance");
    let response = client.get("/badurl").dispatch();

    assert_eq!(response.status(), Status::NotFound);
    assert_eq!(response.content_type(), Some(ContentType::JSON));

    let body = serde_json::from_str(&response.into_string().unwrap()).unwrap();
    validate_error(&body);
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

fn validate_error(instance: &Value) {
    let compiled_schema = get_schema("response-failure.json");
    assert_valid(instance, &compiled_schema);
    let status = instance.get("status")
        .expect("Status property present");
    assert_eq!(status, "failure");
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
