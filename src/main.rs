mod cli;
mod color;
mod decode;
mod http;
mod mime;

use crate::{
    cli::{Args, ParseResult, show_help, show_version},
    color::Color,
    http::{Request, Response},
};
use std::{
    io::{self, Error, ErrorKind},
    net::{TcpListener, TcpStream},
    path::PathBuf,
    process::{self, exit},
    thread,
};

fn canonical_path(base: &PathBuf, request_path: String) -> io::Result<PathBuf> {
    match base.join(&request_path[1..]).canonicalize() {
        Ok(canonicalized) => {
            if canonicalized.starts_with(base) {
                Ok(canonicalized)
            } else {
                Err(Error::new(
                    ErrorKind::PermissionDenied,
                    "Not allowed to escape base directory.",
                ))
            }
        }
        Err(_) => Err(Error::new(ErrorKind::NotFound, "Path does not exist.")),
    }
}

fn handle_connection(mut stream: TcpStream, base: PathBuf) -> io::Result<usize> {
    let request = match Request::get(&mut stream) {
        Ok(request) => request,
        Err(e) => {
            eprintln!("{}: {}", "400 Bad Request".red(), e);
            return Response::error(400).send_to(&mut stream);
        }
    };

    if request.method.is_empty() && request.path.is_empty() {
        // ignore empty requests
        return Ok(0);
    }

    println!("{} {}", request.method.cyan(), request.path.yellow());

    if &request.method != "GET" {
        println!(
            "{}: Requested Http Method {} is not supported.",
            "405 Method Not Allowed".red(),
            request.method
        );
        return Response::error(405).send_to(&mut stream);
    }

    match canonical_path(&base, request.path) {
        Ok(path) => {
            if path.is_dir() {
                let index = path.join("index.html");
                if index.exists() {
                    Response::file(&index).send_to(&mut stream)
                } else {
                    let base = base.to_str().unwrap();
                    // show_dir(&mut stream, base, path)
                    Response::directory(base, &path).send_to(&mut stream)
                }
            } else {
                // send_file(&mut stream, path.as_path())
                Response::file(&path).send_to(&mut stream)
            }
        }
        Err(e) => {
            eprintln!("{}: {}", "404 Not Found".red(), e);
            Response::error(404).send_to(&mut stream)
        }
    }
}

fn main() {
    let args = Args::parse();
    let args = match args {
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
    };

    let port = args.port;
    let base_path = args.path;

    println!("{} {}", "Rup version:".yellow(), cli::VERSION.green());
    println!(
        "{} {}:{}",
        "Starting server".yellow(),
        "on http://localhost".green(),
        port
    );

    let listener = TcpListener::bind(format!("0.0.0.0:{port}")).unwrap_or_else(|e| {
        eprintln!("{}", "Couldn't bind the port".bright_red());
        eprintln!("{e}");
        process::exit(1);
    });
    println!(
        "{} {}",
        "Serving ".yellow(),
        base_path.to_str().unwrap().green()
    );
    println!("Hit Ctrl+C to exit.\n");
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let base_path = base_path.clone();
                thread::spawn(move || match handle_connection(stream, base_path) {
                    Ok(_) => {}
                    Err(e) => eprintln!("{e}"),
                });
            }
            Err(e) => {
                eprintln!("failed: {e}");
            }
        }
    }
}
