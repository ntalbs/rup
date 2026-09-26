mod cli;
mod decode;
mod http;
mod mime;

use crate::{
    cli::Args,
    http::{Request, Response},
};
use coloring::Color;
use std::{
    env, io,
    net::{TcpListener, TcpStream},
    path::PathBuf,
    process, thread,
};

fn handle_connection(mut stream: TcpStream, base: PathBuf) -> io::Result<usize> {
    let request = match Request::get(&mut stream) {
        Ok(request) => request,
        Err(e) => {
            return Response::error(400, &e).send_to(&mut stream);
        }
    };

    if request.method.is_empty() && request.path.is_empty() {
        // ignore empty requests
        return Ok(0);
    }

    println!("{} {}", request.method.cyan(), request.path.yellow());

    if &request.method != "GET" {
        println!(
            "Requested Http Method: {} is not supported.",
            request.method
        );
        return Response::error(405, "Method not allowed").send_to(&mut stream);
    }

    let path = match base.join(&request.path[1..]).canonicalize() {
        Ok(canonicalized) => {
            if canonicalized.starts_with(&base) {
                canonicalized
            } else {
                return Response::error(404_u16, "Requested path does not exist.")
                    .send_to(&mut stream);
            }
        }
        Err(_) => {
            return Response::error(404_u16, "Requested path does not exist.").send_to(&mut stream);
        }
    };

    if !path.exists() {
        Response::error(404_u16, "Requested path does not exist.").send_to(&mut stream)
    } else if path.is_dir() {
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

fn main() {
    let args: Vec<String> = env::args().collect();
    let args = Args::parse(&args);
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
