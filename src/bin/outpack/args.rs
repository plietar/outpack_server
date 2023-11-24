#[derive(clap::Parser, Debug)]
pub struct Args {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(clap::Subcommand, Debug)]
pub enum Command {
    /// Initialize a new outpack repository
    Init {
        path: String,

        #[arg(long)]
        path_archive: Option<String>,

        #[arg(long)]
        use_file_store: bool,

        #[arg(long)]
        require_complete_tree: bool,
    },
    /// Search for a packet in a repository
    Search {
        #[arg(short, long)]
        root: String,
        query: String,
    },
    /// Parse an outpack query, without evaluating it
    Parse { query: String },
    /// Start the outpack API server
    ApiServer {
        #[arg(short, long)]
        root: String,
    },
}
