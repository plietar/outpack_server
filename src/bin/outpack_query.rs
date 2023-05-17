extern crate core;

use getopts::Options;
use std::{env, process};

fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} [options]", program);
    print!("{}", opts.usage(&brief));
}

fn parse_args(args: &[String]) -> (String, String) {
    let program = args[0].clone();
    let mut opts = Options::new();
    opts.reqopt("q", "query", "outpack query (required)", "latest");
    opts.reqopt("r", "root", "outpack root path (required)", ".");
    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => {
            print_usage(&program, opts);
            panic!("{}", f.to_string())
        }
    };
    (matches.opt_str("r").unwrap(), matches.opt_str("q").unwrap())
}

fn main() {
    let args = env::args().collect::<Vec<_>>();
    let (root, query) = parse_args(&args);
    let result = outpack::query::run_query(&root, query);
    match result {
        Ok(res) => {
            println!("{}", res)
        }
        Err(e) => {
            eprintln!("{}", e);
            process::exit(1);
        }
    }
}
