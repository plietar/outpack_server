extern crate core;

use getopts::Options;
use std::env;

use outpack::init::outpack_init;

struct InitOptions {
    path: String,
    path_archive: Option<String>,
    use_file_store: bool,
    require_complete_tree: bool,
}

fn usage(program: &str, opts: &Options) -> String {
    let brief = format!("Usage: {} [options]", program);
    opts.usage(&brief).to_string()
}

fn parse_args(args: &[String]) -> Result<InitOptions, String> {
    let program = args[0].clone();
    let mut opts = Options::new();
    opts.optopt("", "path-archive", "path to the archive", "path");
    opts.optflag("", "use-file-store", "use a file store");
    opts.optflag("", "require-complete-tree", "require a complete tree");
    opts.reqopt("", "path", "path to the outpack path", "path");
    opts.optflag("h", "help", "print this help");

    let res = opts.parse(&args[1..]).map_err(|_| usage(&program, &opts))?;
    if res.opt_present("h") {
        return Err(usage(&program, &opts));
    }
    let use_file_store = res.opt_present("use-file-store");
    let require_complete_tree = res.opt_present("require-complete-tree");
    let path = res.opt_str("path").unwrap();
    let path_archive = if res.opt_present("path-archive") {
        Some(res.opt_str("path-archive").unwrap())
    } else {
        None
    };
    Ok(InitOptions {
        path,
        path_archive,
        use_file_store,
        require_complete_tree,
    })
}

fn do_init(opts: InitOptions) -> Result<(), String> {
    outpack_init(
        &opts.path,
        opts.path_archive,
        opts.use_file_store,
        opts.require_complete_tree,
    )
    .map_err(|e| e.to_string())
}

fn main() -> Result<(), String> {
    let args = env::args().collect::<Vec<_>>();
    let result = parse_args(&args)?;
    do_init(result)?;
    Ok(())
}
