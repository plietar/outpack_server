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
        "schema_version": "0.1.4"
   },
   "errors": null
}
```

## License
MIT © Imperial College of Science, Technology and Medicine
