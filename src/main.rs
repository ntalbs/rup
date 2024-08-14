mod cli;
mod decode;
mod http;
mod mime;

use crate::{
    cli::Args,
    http::{Request, Response},
};
use colorust::Color;
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

    println!("{} {}", &request.method.cyan(), &request.path.yellow());

    if &request.method != "GET" {
        println!(
            "Requested Http Method: {} is not supported.",
            &request.method
        );
        return Response::error(405, "Method not allowed").send_to(&mut stream);
    }

    let mut path = base.clone();
    if request.path != "/" {
        path.push(&request.path[1..]);
    }

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
        args.path.canonicalize().unwrap().to_str().unwrap().green()
    );
    println!("Hit Ctrl+C to exit.\n");
    let base_path = args.path;
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let path = base_path.clone();
                thread::spawn(move || match handle_connection(stream, path) {
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
