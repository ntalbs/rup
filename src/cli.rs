use std::process::exit;

use crate::color::{Color, Style};

const VERSION: &str = env!("CARGO_PKG_VERSION");
const DEFAULT_PORT: u16 = 3000;

fn show_version() {
    println!("rup {VERSION}");
}

fn show_help() {
    fn print_opt(opt: &str, val: &str, desc: &str) {
        println!("  {:25} {:8} {}", opt.bright_white(), val, desc);
    }
    println!("A simple command-line static http server");
    println!();
    println!(
        "{}: {} {}",
        "Usage".underline().bright_white(),
        "rup".bright_white(),
        "[OPTIONS]"
    );
    println!();
    println!("{}:", "Options".underline().bright_white());
    print_opt(
        "-p, --port",
        "<PORT>",
        &format!("[default: {DEFAULT_PORT}]"),
    );
    print_opt("-h, --help", "", "Print help information");
    print_opt("-V, --version", "", "Print version information");
}

pub(crate) struct Args {
    pub port: u16,
}

enum ParseResult {
    Args(Args),
    Help,
    Version,
}

struct ParseError {
    reason: String,
}

impl Args {
    fn parse_internal(args: &[String]) -> Result<ParseResult, ParseError> {
        let mut ret = Self { port: DEFAULT_PORT };
        let mut args_iter = args.iter();

        loop {
            if let Some(arg) = args_iter.next() {
                match arg.as_str() {
                    "-p" | "--port" => {
                        if let Some(port_str) = args_iter.next() {
                            ret.port = match port_str.parse() {
                                Ok(port) => port,
                                Err(e) => {
                                    let reason = format!(
                                        "Invalid value '{}' for '--port <PORT>': {}",
                                        port_str, e
                                    );
                                    return Err(ParseError { reason })
                                }
                            }
                        } else {
                            let reason = "error: The argument '--port <PORT>' requires a value but none was supplied".to_string();
                            return Err(ParseError { reason });
                        }
                    }
                    "-V" | "--version" => {
                        return Ok(ParseResult::Version);
                    }
                    "-h" | "--help" => {
                        return Ok(ParseResult::Help);
                    }
                    _ => {
                        let mesg = format!("error: Found argument '{}' which wasn't expected, or isn't valid in this context", arg);
                        return Err(ParseError { reason: mesg });
                    }
                }
            } else {
                return Ok(ParseResult::Args(ret));
            }
        }
    }

    pub(crate) fn parse(args: &[String]) -> Self {
        match Self::parse_internal(&args[1..]) {
            Ok(r) => match r {
                ParseResult::Args(a) => return a,
                ParseResult::Help => {
                    show_help();
                    exit(0);
                }
                ParseResult::Version => {
                    show_version();
                    exit(0);
                }
            },
            Err(e) => {
                eprintln!("{}", e.reason);
                exit(1);
            }
        }
    }
}

#[test]
fn test_p() {
    let args = vec!["rup".to_string(), "-p".to_string(), "1024".to_string()];
    let args = Args::parse(&args);
    assert_eq!(args.port, 1024);
}

#[test]
fn test_port() {
    let args = vec!["rup".to_string(), "--port".to_string(), "1024".to_string()];
    let args = Args::parse(&args);
    assert_eq!(args.port, 1024);
}
