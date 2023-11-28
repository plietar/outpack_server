mod args;
use args::{Args, Command};

use clap::Parser;
use outpack::init::outpack_init;
use outpack::query::{parse_query, run_query, query_types::QueryNode};

fn main() -> anyhow::Result<()> {
    let cli = Args::parse();
    match cli.command {
        Command::Init {
            path,
            path_archive,
            use_file_store,
            require_complete_tree,
        } => {
            outpack_init(&path, path_archive, use_file_store, require_complete_tree)?;
        }

        Command::Search { root, query } => {
            let result = run_query(&root, &query)?;
            println!("{}", result);
        }

        Command::ApiServer { root } => {
            let server = outpack::api::api(&root)?;
            rocket::execute(server.launch())?;
        }

        Command::Parse { query, pretty } => {
            let result = parse_query(&query)?;
            if pretty {
                println!("{}", serde_json::to_string_pretty(&result).unwrap());
            } else {
                println!("{}", serde_json::to_string(&result).unwrap());
            }
        }

        Command::Schema => {
            let schema = schemars::schema_for!(QueryNode);
            println!("{}", serde_json::to_string_pretty(&schema).unwrap());
        }
    }
    Ok(())
}
