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

#[allow(unused_must_use)]
async fn start_app(root_path: String) -> Result<(), rocket::Error> {
    match outpack::api::api(root_path) {
        Err(error) => {panic!("{}", error);}
        Ok(api) => {api.launch().await;}
    }
    Ok(())
}

#[rocket::main]
#[allow(unused_must_use)]
async fn main() -> Result<(), rocket::Error> {
    let args = env::args().collect::<Vec<_>>();
    let root = parse_args(&args);
    if let Some(root_path) = root {
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
