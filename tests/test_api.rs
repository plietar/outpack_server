use std::fs;
use std::path::Path;
use rocket::local::blocking::Client;
use rocket::http::{ContentType, Status};
use jsonschema::{Draft, JSONSchema};

#[test]
fn can_get_index() {
    let rocket = outpack_server::api(String::from("tests/example"));
    let client = Client::tracked(rocket).expect("valid rocket instance");
    let response = client.get("/").dispatch();

    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response.content_type(), Some(ContentType::JSON));

    let body = serde_json::from_str(&response.into_string().unwrap()).unwrap();
    validate("root.json", body);
}

#[test]
fn error_if_cant_get_index() {
    let rocket = outpack_server::api(String::from("badlocation"));
    let client = Client::tracked(rocket).expect("valid rocket instance");
    let response = client.get("/").dispatch();

    assert_eq!(response.status(), Status::InternalServerError);
    assert_eq!(response.content_type(), Some(ContentType::JSON));

    let body = serde_json::from_str(&response.into_string().unwrap()).unwrap();
    validate_error(body);
}

fn validate(schema_name: &str, instance: serde_json::Value) {
    let status = instance.get("status")
        .expect("Status property present");
    assert_eq!(status, "success");
    let data = instance.get("data")
        .expect("Data property present");
    let schema_path = Path::new("schema")
        .join(schema_name);
    let schema_as_string = fs::read_to_string(schema_path)
        .expect("Schema file");

    let json_schema = serde_json::from_str(&schema_as_string)
        .expect("Schema is valid json");

    let compiled = JSONSchema::options()
        .with_draft(Draft::Draft7)
        .compile(&json_schema)
        .expect("A valid schema");
    let result = compiled.validate(&data);
    if let Err(errors) = result {
        for error in errors {
            println!("Validation error: {}", error);
            println!("Instance path: {}", error.instance_path);
        }
    }
    assert!(compiled.is_valid(&data));
}

fn validate_error(instance: serde_json::Value) {
    let status = instance.get("status")
        .expect("Status property present");
    assert_eq!(status, "failure");
    let errors = instance.get("errors")
        .expect("Error property present");
}
