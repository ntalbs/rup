use std::{collections::HashMap, process::exit};

use crate::colored::Colorize;

pub(crate) struct Args {
    pub port: u16,
}

impl Args {
    pub(crate) fn parse(args: &[String]) -> Self {
        let map: HashMap<&str, &str> = args[1..]
            .chunks_exact(2)
            .map(|c| (c[0].as_str(), c[1].as_str()))
            .collect();

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
