use std::fs;
use std::io::Read;
use std::path::Path;
use std::sync::Arc;
use rocket::local::blocking::Client;
use rocket::http::{ContentType, Status};
use jsonschema::{Draft, JSONSchema, SchemaResolverError};
use serde_json::Value;
use url::Url;

#[test]
fn can_get_index() {
    let rocket = outpack::api::api(String::from("tests/example"));
    let client = Client::tracked(rocket).expect("valid rocket instance");
    let response = client.get("/").dispatch();

    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response.content_type(), Some(ContentType::JSON));

    let body = serde_json::from_str(&response.into_string().unwrap()).unwrap();
    validate_success("root.json", &body);
}

#[test]
fn error_if_cant_get_index() {
    let rocket = outpack::api::api(String::from("badlocation"));
    let client = Client::tracked(rocket).expect("valid rocket instance");
    let response = client.get("/").dispatch();

    assert_eq!(response.status(), Status::NotFound);
    assert_eq!(response.content_type(), Some(ContentType::JSON));

    let body = serde_json::from_str(&response.into_string().unwrap()).unwrap();
    validate_error(&body, Some("No such file or directory"));
}

#[test]
fn can_get_checksum() {
    let rocket = outpack::api::api(String::from("tests/example"));
    let client = Client::tracked(rocket).expect("valid rocket instance");
    let response = client.get("/checksum").dispatch();

    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response.content_type(), Some(ContentType::JSON));

    let body = serde_json::from_str(&response.into_string().unwrap()).unwrap();
    validate_success("hash.json", &body);

    let response = client.get("/checksum?alg=md5").dispatch();

    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response.content_type(), Some(ContentType::JSON));

    let response_string = &response.into_string().unwrap();
    let body = serde_json::from_str(response_string).unwrap();
    validate_success("hash.json", &body);
    assert!(response_string.contains("md5"))
}

#[test]
fn can_list_location_metadata() {
    let rocket = outpack::api::api(String::from("tests/example"));
    let client = Client::tracked(rocket).expect("valid rocket instance");
    let response = client.get("/location/metadata").dispatch();

    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response.content_type(), Some(ContentType::JSON));

    let body: Value = serde_json::from_str(&response.into_string().unwrap()).unwrap();
    print!("{}", body);
    validate_success("location.json", &body);

    let entries = body.get("data").unwrap().as_array().unwrap();
    assert_eq!(entries.len(), 3);

    assert_eq!(entries[0].get("packet").unwrap().as_str().unwrap(), "20170817-164847-7574883b");
    assert_eq!(entries[0].get("time").unwrap().as_f64().unwrap(), 1662480556.1778);
    assert_eq!(entries[0].get("hash").unwrap().as_str().unwrap(),
               "sha256:af3c863f96898c6c88cee4daa1a6d6cfb756025e70059f5ea4dbe4d9cc5e0e36");

    assert_eq!(entries[1].get("packet").unwrap().as_str().unwrap(), "20170818-164043-7cdcde4b");
    assert_eq!(entries[2].get("packet").unwrap().as_str().unwrap(), "20170818-164830-33e0ab01");
}

#[test]
fn handles_location_metadata_errors() {
    let rocket = outpack::api::api(String::from("tests/bad-example"));
    let client = Client::tracked(rocket).expect("valid rocket instance");
    let response = client.get("/location/metadata").dispatch();
    assert_eq!(response.status(), Status::InternalServerError);
    assert_eq!(response.content_type(), Some(ContentType::JSON));

    let body = serde_json::from_str(&response.into_string().unwrap()).unwrap();
    validate_error(&body, Some("missing field `packet`"));
}

#[test]
fn can_list_metadata() {
    let rocket = outpack::api::api(String::from("tests/example"));
    let client = Client::tracked(rocket).expect("valid rocket instance");
    let response = client.get("/metadata").dispatch();

    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response.content_type(), Some(ContentType::JSON));

    let body: Value = serde_json::from_str(&response.into_string().unwrap()).unwrap();
    print!("{}", body);
    validate_success("list.json", &body);

    let entries = body.get("data").unwrap().as_array().unwrap();
    assert_eq!(entries.len(), 3);

    assert_eq!(entries[0].get("id").unwrap().as_str().unwrap(), "20170817-164847-7574883b");
    assert_eq!(entries[0].get("name").unwrap().as_str().unwrap(), "modup-201707-queries1");
    assert_eq!(entries[0].get("parameters").unwrap()
                   .as_object().unwrap().get("disease").unwrap().as_str().unwrap(), "YF");
    assert_eq!(entries[0].get("custom").unwrap()
                   .as_object().unwrap().get("orderly").unwrap()
                   .as_object().unwrap().get("displayname").unwrap().as_str().unwrap(),
               "Modified Update");

    assert_eq!(entries[1].get("id").unwrap().as_str().unwrap(), "20170818-164847-7574883c");
    assert_eq!(entries[2].get("id").unwrap().as_str().unwrap(), "20180818-164847-54699abf");
}

#[test]
fn can_list_metadata_from_date() {
    let rocket = outpack::api::api(String::from("tests/example"));
    let client = Client::tracked(rocket).expect("valid rocket instance");
    let response = client.get("/metadata?from=20170818-170000").dispatch();

    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response.content_type(), Some(ContentType::JSON));

    let body: Value = serde_json::from_str(&response.into_string().unwrap()).unwrap();
    print!("{}", body);
    validate_success("list.json", &body);

    let entries = body.get("data").unwrap().as_array().unwrap();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].get("id").unwrap().as_str().unwrap(), "20180818-164847-54699abf");
}

#[test]
fn handles_metadata_errors() {
    let rocket = outpack::api::api(String::from("tests/bad-example"));
    let client = Client::tracked(rocket).expect("valid rocket instance");
    let response = client.get("/metadata").dispatch();
    assert_eq!(response.status(), Status::InternalServerError);
    assert_eq!(response.content_type(), Some(ContentType::JSON));

    let body = serde_json::from_str(&response.into_string().unwrap()).unwrap();
    validate_error(&body, Some("missing field `name`"));
}

#[test]
fn can_get_metadata_json() {
    let rocket = outpack::api::api(String::from("tests/example"));
    let client = Client::tracked(rocket).expect("valid rocket instance");
    let response = client.get("/metadata/20170817-164847-7574883b/json").dispatch();

    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response.content_type(), Some(ContentType::JSON));

    let body: Value = serde_json::from_str(&response.into_string().unwrap()).unwrap();
    validate_success("metadata.json", &body);
}

#[test]
fn can_get_metadata_text() {
    let rocket = outpack::api::api(String::from("tests/example"));
    let client = Client::tracked(rocket).expect("valid rocket instance");
    let response = client.get("/metadata/20170817-164847-7574883b/text").dispatch();

    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response.content_type(), Some(ContentType::Text));

    let expected = fs::File::open(Path::new("tests/example/.outpack/metadata/20170817-164847-7574883b"))
        .unwrap();
    let result: Value = serde_json::from_str(&response.into_string().unwrap()[..]).unwrap();
    let expected: Value = serde_json::from_reader(expected).unwrap();
    assert_eq!(result, expected);
}

#[test]
fn returns_404_if_packet_not_found() {
    let rocket = outpack::api::api(String::from("tests/example"));
    let client = Client::tracked(rocket).expect("valid rocket instance");
    let response = client.get("/metadata/bad-id/json").dispatch();

    assert_eq!(response.status(), Status::NotFound);
    assert_eq!(response.content_type(), Some(ContentType::JSON));

    let body = serde_json::from_str(&response.into_string().unwrap()).unwrap();
    validate_error(&body, Some("packet with id 'bad-id' does not exist"))
}

#[test]
fn can_get_file() {
    let rocket = outpack::api::api(String::from("tests/example"));
    let client = Client::tracked(rocket).expect("valid rocket instance");
    let hash = "sha256:b189579a9326f585d308304bd9e03326be5d395ac71b31df359ab8bac408d248";
    let response = client.get(format!("/file/{}", hash)).dispatch();

    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response.content_type(), Some(ContentType::Binary));

    let path = Path::new("tests/example/.outpack/files/sha256/b1/")
        .join("89579a9326f585d308304bd9e03326be5d395ac71b31df359ab8bac408d248");
    let mut file = fs::File::open(&path)
        .unwrap();
    let metadata = fs::metadata(&path).unwrap();
    let mut buffer = vec![0; metadata.len() as usize];

    file.read(&mut buffer)
        .unwrap();

    assert_eq!(response.into_bytes().unwrap(), buffer);
}

#[test]
fn returns_404_if_file_not_found() {
    let rocket = outpack::api::api(String::from("tests/example"));
    let client = Client::tracked(rocket).expect("valid rocket instance");
    let hash = "sha256:123456";
    let response = client.get(format!("/file/{}", hash)).dispatch();

    assert_eq!(response.status(), Status::NotFound);
    assert_eq!(response.content_type(), Some(ContentType::JSON));

    let body = serde_json::from_str(&response.into_string().unwrap()).unwrap();
    validate_error(&body, Some("hash 'sha256:123456' not found"))
}

#[test]
fn catches_arbitrary_404() {
    let rocket = outpack::api::api(String::from("tests/example"));
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
