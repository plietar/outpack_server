extern crate core;

use getopts::Options;
use std::env;

fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} [options]", program);
    print!("{}", opts.usage(&brief));
}

fn parse_args(args: &[String]) -> Option<String> {
    let program = args[0].clone();
    let mut opts = Options::new();
    opts.reqopt("r", "root", "outpack root path (required)", ".");
    let matches = match opts.parse(&args[1..]) {
        Ok(m) => { m }
        Err(f) => {
            print_usage(&program, opts);
            panic!("{}", f.to_string())
        }
    };
    Some(matches.opt_str("r").unwrap())
}

fn main() {
    let args = env::args().collect::<Vec<_>>();
    let root = parse_args(&args);
    if let Some(root_path) = root {
        let _cfg = outpack::config::read_config(&root_path)
            .unwrap_or_else(|error| {
                panic!("Could not open outpack root at {}: {:?}",
                       root_path, error);
            });
        println!("Root '{}' was opened successfully", root_path);
    }
}
