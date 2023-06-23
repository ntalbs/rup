use std::{collections::HashMap, process::exit};

use crate::color::{Color, Style};

const VERSION: &str = env!("CARGO_PKG_VERSION");
const DEFAULT_PORT: u16 = 3000;

pub(crate) struct Args {
    pub port: u16,
}

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

impl Args {
    pub(crate) fn parse(args: &[String]) -> Self {
        let map: HashMap<&str, &str> = args[1..]
            .chunks_exact(2)
            .map(|c| (c[0].as_str(), c[1].as_str()))
            .collect();

        if map.contains_key("-V") || map.contains_key("--version") {
            show_version();
            exit(0);
        }

        if map.contains_key("-h") || map.contains_key("--help") {
            show_help();
            exit(0);
        }

        let port: u16 = map
            .get("--port")
            .or_else(|| map.get("-p"))
            .unwrap_or(&"80")
            .to_string()
            .parse()
            .unwrap_or_else(|e| {
                eprint!("{}: ", "error".red());
                eprint!("Invalid value for '");
                eprint!("{}': ", "--port <PORT>".yellow());
                eprintln!("{}", e);
                exit(1);
            });
        Self { port }
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
