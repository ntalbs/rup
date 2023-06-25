use std::process::exit;

use crate::color::{Color, Style};

pub(crate) const VERSION: &str = env!("CARGO_PKG_VERSION");
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
        "{}: {} [OPTIONS]",
        "Usage".underline().bright_white(),
        "rup".bright_white(),
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

#[derive(Debug, PartialEq)]
pub(crate) struct Args {
    pub port: u16,
}

#[derive(Debug, PartialEq)]
enum ParseResult {
    Args(Args),
    Help,
    Version,
}

struct ParseError {
    reason: String,
}

struct ArgsParser<'a> {
    tokens: &'a [String],
    current: usize,
}

impl<'a> ArgsParser<'a> {
    fn new(tokens: &'a [String]) -> Self {
        Self { tokens, current: 0 }
    }

    fn advance(&mut self) -> &String {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.tokens.len()
    }

    fn peek(&self) -> &String {
        &self.tokens[self.current]
    }

    fn previous(&self) -> &String {
        &self.tokens[self.current - 1]
    }

    fn parse(&mut self) -> Result<ParseResult, ParseError> {
        let mut ret = Args { port: DEFAULT_PORT };

        while !self.is_at_end() {
            let token = self.advance();
            match token.as_str() {
                "-p" | "--port" => {
                    let port_str = self.peek();
                    if !port_str.starts_with('-') {
                        ret.port = match port_str.parse() {
                            Ok(port) => port,
                            Err(e) => {
                                let reason = format!(
                                    "Invalid value '{}' for '{}': {}",
                                    port_str.yellow(),
                                    "--port <PORT>".yellow(),
                                    e
                                );
                                return Err(ParseError { reason });
                            }
                        };
                        self.advance();
                    } else {
                        let reason = format!(
                            "{}: The argument '{}' requires a value but none was supplied",
                            "error".bright_red(),
                            "--port <PORT>".yellow()
                        );
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
                    let reason = format!("{}: Found argument '{}' which wasn't expected, or isn't valid in this context", "error".bright_red(), token.yellow());
                    return Err(ParseError { reason });
                }
            }
        }
        Ok(ParseResult::Args(ret))
    }
}

impl Args {
    pub(crate) fn parse(args: &[String]) -> Self {
        let mut arg_parser = ArgsParser::new(&args[1..]);

        match arg_parser.parse() {
            Ok(r) => match r {
                ParseResult::Args(a) => a,
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

#[test]
fn test_version() {
    let args = vec!["--version".to_string(), "-p".to_string()];
    if let Ok(result) = ArgsParser::new(&args).parse() {
        assert_eq!(result, ParseResult::Version);
    } else {
        assert!(false);
    }
}

#[test]
fn test_help() {
    let args = vec!["--help".to_string(), "-p".to_string()];
    if let Ok(result) = ArgsParser::new(&args).parse() {
        assert_eq!(result, ParseResult::Help);
    } else {
        assert!(false);
    }
}
