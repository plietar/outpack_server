extern crate core;

use getopts::Options;
use std::env;
use std::path::Path;

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

#[allow(unused_must_use)]
async fn start_app(root_path: String) -> Result<(), rocket::Error> {
    if !Path::new(&root_path).join(".outpack").exists() {
        panic!("Outpack root not found at {}", root_path)
    }
    outpack_server::api(root_path).launch().await;
    Ok(())
}

#[rocket::main]
#[allow(unused_must_use)]
async fn main() -> Result<(), rocket::Error> {
    let args = env::args().collect::<Vec<_>>();
    let root = parse_args(&args);
    if root.is_some() {
        let root_path = root.unwrap();
        start_app(root_path).await;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_parse_args() {
        let root = parse_args(&[String::from("program"), String::from("--root"), String::from("test")]).unwrap();
        assert_eq!(root, "test");

        let root = parse_args(&[String::from("program"), String::from("-r"), String::from("test")]).unwrap();
        assert_eq!(root, "test");
    }

    #[test]
    #[should_panic]
    fn panics_if_args_not_valid() {
        parse_args(&[String::from("program")]);
    }

    #[rocket::async_test]
    #[should_panic]
    #[allow(unused_must_use)]
    async fn panics_if_outpack_not_found() {
        start_app(String::from("badpath")).await;
    }
}
