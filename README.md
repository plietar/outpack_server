# outpack_server
[![Project Status: Concept – Minimal or no implementation has been done yet, or the repository is only intended to be a limited example, demo, or proof-of-concept.](https://www.repostatus.org/badges/latest/concept.svg)](https://www.repostatus.org/#concept)

Rust implementation of the `outpack` HTTP API.

## Usage
Start with `cargo run -- --root <path>`. Or build the binary
with `cargo build` and run directly with `target/debug/outpack_server run --root <path>`

E.g.

```cargo run -- --root tests/example```

## Schema
The outpack schema is imported into this package by running `./scripts/import_schema`,
and needs to be kept manually up to date by re-running that script as needed.

## Tests
Run all tests with `cargo test`.

## GET /

```
{
   "status": "succcess",
   "data": {
        "schema_version": "0.0.1"
   },
   "errors": null
}
```

## GET /metadata/list

```
{
    "status": "success",
    "errors": null,
    "data": [
        {
            "packet": "20220812-155808-c873e405",
            "time": "2022-08-12 15:58:08",
            "hash": "sha256:df6edb3d6cd50f5aec9308a357111592cde480f45a5f46341877af21ae30d93e"
        },
        {
            "packet": "20220812-155808-d5747caf",
            "time": "2022-08-12 15:58:08",
            "hash": "sha256:edc70ef51e69f2cde8238142af29a9419bb27c94b320b87e88f617dfc977a46b"
        },
        {
            "packet": "20220812-155808-dbd3ce81",
            "time": "2022-08-12 15:58:08",
            "hash": "sha256:a7da8c3464a2da4722b9d15daa98eb13f4f8c1949c6d00100428b2e9d0668f29"
        },
        {
            "packet": "20220812-155808-e21bc5fc",
            "time": "2022-08-12 15:58:08",
            "hash": "sha256:df1b91aaf3393483515ac61596aa35117891eacc533a55ec2f4759d0036514f9"
        }
    ]
}
```

## License
MIT © Imperial College of Science, Technology and Medicine
