# outpack

[![Project Status: Concept – Minimal or no implementation has been done yet, or the repository is only intended to be a limited example, demo, or proof-of-concept.](https://www.repostatus.org/badges/latest/concept.svg)](https://www.repostatus.org/#concept)

Rust implementation of `outpack`. This crate provides two binaries:

* `outpack`: an outpack CLI, designed to interact with any outpack archive
* `outpack_server`: an HTTP server, implementing the outpack API

## Cli usage

```
cargo run --bin outpack -- --root <path>
```

## Server usage

Start with `cargo run --bin outpack_server -- --root <path>`. Or build the binary
with `cargo build` and run directly with `target/debug/outpack_server run --root <path>`

E.g.

```
cargo run --bin outpack_server -- --root tests/example
```

## Usage of docker image

```
docker run --name outpack_server -v /full/path/to/root:/outpack -p 8000:8000 -d mrcide/outpack_server:main
```

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

## GET /checksum

Returns hash of all current packet ids, ordered alphanumerically and concatenated. This will use the hashing algorithm specified 
in the `outpack` config, unless a query parameter specifying an alternative is passed: 
e.g. `/checksum?alg=md5`. 

```
{
   "status": "succcess",
   "data": "md5:117723186364b4b409081b1bd347d406"
   "errors": null
}
```

## GET /location/metadata

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

## GET /metadata

Returns a list of (truncated) packet metadata. 
Accepts an optional query parameter `known_since` specifying a Unix epoch time 
from which to return results. This will filter packets by the `time` property of the 
location metadata, i.e. the point at which they were inserted into the index.
e.g. `/metadata?known_since=1683117048`. 

```
{
    "status": "success",
    "errors": null,
    "data": [
        {
            "id": "20220812-155808-c873e405",
            "name": "depends",
            "custom": { "orderly": { "display": "Report with dependencies" }}
            "parameters": null
        },
        {
            "id": "20220812-155808-d5747caf",
            "name": "params",
            "custom": { "orderly": { "display": "Report with parameters" }},
            "parameters": { "alpha": 1 }
        }
    ]
}
```


## GET /metadata/\<id\>/json

```
{
  "status": "success",
  "errors": null,
  "data": {
    "custom": null,
    "depends": [],
    "files": [
      {
        "hash": "sha256:e9aa9f2212aba6fba4464212800a2927afa02eda688cf13131652da307e3d7c1",
        "path": "orderly.yml",
        "size": 955
      },
      {
        "hash": "sha256:11a2cd93493defa673b198d5be7a180cef7b133baaacc046923e1e2da77c6f75",
        "path": "modified_update.R",
        "size": 1133
      },
      {
        "hash": "sha256:c4d4c95af9da912f2f20c65a0502c7da19a5712767a39e07a2dd1ea7fcb615b0",
        "path": "R/util.R",
        "size": 2757
      }
    ],
    "id": "20170818-164043-7cdcde4b",
    "name": "modup-201707",
    "parameters": null,
    "schema_version": "0.0.1",
    "script": [
      "modified_update.R"
    ],
    "session": {
      "packages": [
        {
          "attached": true,
          "package": "RcppRoll",
          "version": "0.2.2"
        },
        {
          "attached": false,
          "package": "Rcpp",
          "version": "0.12.12"
        },
        {
          "attached": false,
          "package": "ids",
          "version": "1.0.1"
        }
      ],
      "platform": {
        "os": "Debian GNU/Linux 9 (stretch)",
        "system": "x86_64, linux-gnu",
        "version": "R version 3.4.0 (2017-04-21)"
      }
    },
    "time": {
      "end": 1503074545.8687,
      "start": 1503074545.8687
    }
  }
}
```

## GET /metadata/\<id\>/text

Returns the same as `GET /metadata/<id>/json` but just the data as plain text.

## GET /file/\<hash\>

Downloads the file with the provided hash. 404 if it doesn't exist.

## License

MIT © Imperial College of Science, Technology and Medicine
