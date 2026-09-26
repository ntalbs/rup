use coloring::{Color, Style};
use std::{env, path::PathBuf};

pub(crate) const VERSION: &str = env!("CARGO_PKG_VERSION");
const DEFAULT_PORT: u16 = 3000;

pub(crate) fn show_version() {
    println!("rup {VERSION}");
}

pub(crate) fn show_help() {
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
        &format!("Port to use [default: {DEFAULT_PORT}]"),
    );
    print_opt("-r, --root", "<PATH>", "Base directory [default: \".\"]");
    print_opt("-h, --help", "", "Print help information");
    print_opt("-V, --version", "", "Print version information");
}

#[derive(Debug, PartialEq)]
pub(crate) struct Args {
    pub port: u16,
    pub path: PathBuf,
}

#[derive(Debug, PartialEq)]
pub(crate) enum ParseResult {
    Args(Args),
    Help,
    Version,
}

pub(crate) struct ParseError {
    pub(crate) reason: String,
}

struct ArgsParser<'a> {
    tokens: &'a [String],
    current: usize,
}

fn parse_port(port_str: &str) -> Result<u16, ParseError> {
    if let Ok(port) = port_str.parse()
        && (1..=u16::MAX).contains(&port)
    {
        Ok(port)
    } else {
        Err(ParseError {
            reason: format!(
                "Invalid value '{}' for '{}'",
                port_str.yellow(),
                "--port <PORT>".yellow()
            ),
        })
    }
}

fn parse_root_dir(root: &str) -> Result<PathBuf, ParseError> {
    if let Ok(path) = PathBuf::from(root).canonicalize() {
        Ok(path)
    } else {
        Err(ParseError {
            reason: format!(
                "{}: The specified path '{}' does not exist.",
                "Error".bright_red(),
                root.yellow()
            ),
        })
    }
}

impl<'a> ArgsParser<'a> {
    fn new(tokens: &'a [String]) -> Self {
        Self { tokens, current: 0 }
    }

    fn next_token(&mut self) -> Option<&String> {
        if self.is_at_end() {
            return None;
        }
        self.current += 1;
        Some(&self.tokens[self.current - 1])
    }

    fn next_value(&mut self, flag: &str) -> Result<&String, ParseError> {
        if let Some(next) = self.tokens.get(self.current)
            && !next.starts_with('-')
        {
            self.current += 1;
            Ok(next)
        } else {
            Err(ParseError {
                reason: format!(
                    "{}: The argument '{}' requires a value but none was supplied",
                    "Error".bright_red(),
                    flag.yellow()
                ),
            })
        }
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.tokens.len()
    }

    fn parse(&mut self) -> Result<ParseResult, ParseError> {
        let mut ret = Args {
            port: DEFAULT_PORT,
            path: PathBuf::from(".").canonicalize().unwrap(),
        };

        while let Some(token) = self.next_token() {
            match token.as_str() {
                "-p" | "--port" => {
                    let port_str = self.next_value("--port <PORT>")?;
                    ret.port = parse_port(port_str)?;
                }
                "-r" | "--root" => {
                    let root = self.next_value("--root <ROOT>")?;
                    ret.path = parse_root_dir(root)?;
                }
                "-V" | "--version" => {
                    return Ok(ParseResult::Version);
                }
                "-h" | "--help" => {
                    return Ok(ParseResult::Help);
                }
                _ => {
                    let reason = format!(
                        "{}: Found argument '{}' which wasn't expected, or isn't valid in this context",
                        "Error".bright_red(),
                        token.yellow()
                    );
                    return Err(ParseError { reason });
                }
            }
        }
        Ok(ParseResult::Args(ret))
    }
}

impl Args {
    pub(crate) fn parse() -> Result<ParseResult, ParseError> {
        let args: Vec<String> = env::args().collect();
        let mut arg_parser = ArgsParser::new(&args[1..]);
        arg_parser.parse()
    }
}

#[cfg(test)]
mod test {
    use p_test::p_test;

    use crate::cli::{Args, ArgsParser, ParseResult};

    #[p_test(
        (vec![], Args { port: 3000, path: ".".into() }),
        (vec!["-p", "1024"], Args { port: 1024, path: ".".into() }),
        (vec!["--port", "1024"], Args { port: 1024, path: ".".into() }),
    )]
    fn arg_parse_test(input: Vec<&str>, expected: Args) {
        let input: Vec<String> = input.into_iter().map(String::from).collect();
        if let Ok(actual) = ArgsParser::new(&input).parse() {
            match actual {
                ParseResult::Args(Args { port, path }) => {
                    assert_eq!(port, expected.port);
                    assert_eq!(path, expected.path.canonicalize().unwrap());
                }
                _ => panic!(),
            }
        };
    }

    #[p_test(
        (vec!["-p"]),
        (vec!["-r"]),
        (vec!["-port"]),
        (vec!["-root"]),
        (vec!["-p", "-r"]),
        (vec!["-p", "-x"]),
        (vec!["-x"]),
        (vec!["-p", "0"]),
        (vec!["-p", "65536"]),
        (vec!["--port", "-1024"]),
    )]
    fn arg_parse_err_test(input: Vec<&str>) {
        let input: Vec<String> = input.into_iter().map(String::from).collect();
        assert!(ArgsParser::new(&input).parse().is_err());
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
}
