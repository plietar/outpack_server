extern crate core;

use getopts::Options;
use std::{env, process::ExitCode};

use outpack::query::QueryError;

fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} [options]", program);
    print!("{}", opts.usage(&brief));
}

enum Args {
    Parse { query: String },
    Eval { query: String, root: String },
}

fn parse_args(args: &[String]) -> Args {
    let program = args[0].clone();
    let mut opts = Options::new();
    opts.reqopt("q", "query", "outpack query (required)", "latest");
    opts.optopt("r", "root", "outpack root path", ".");
    opts.optflag("", "parse-only", "parse the query without running it");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => {
            print_usage(&program, opts);
            panic!("{}", f.to_string())
        }
    };

    if matches.opt_present("parse-only") && matches.opt_present("root") {
        panic!("--parse-only and --root are mutually exclusive");
    }
    if !matches.opt_present("parse-only") && !matches.opt_present("root") {
        panic!("Either --parse-only or --root are required");
    }

    if matches.opt_present("parse-only") {
        Args::Parse {
            query: matches.opt_str("q").unwrap(),
        }
    } else {
        Args::Eval {
            query: matches.opt_str("q").unwrap(),
            root: matches.opt_str("r").unwrap(),
        }
    }
}

fn run() -> Result<(), QueryError> {
    let args = env::args().collect::<Vec<_>>();
    match parse_args(&args) {
        Args::Eval { query, root } => {
            let result = outpack::query::run_query(&root, &query)?;
            println!("{}", result);
        }
        Args::Parse { query } => {
            let result = outpack::query::parse_query(&query)?;
            println!("{:?}", result);
        }
    };

    Ok(())
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("{}", err);
            ExitCode::FAILURE
        }
    }
}
