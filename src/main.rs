mod cli;
mod decode;
mod http;
mod mime;

use crate::{cli::Args, http::*};
use colorust::Color;
use std::{
    env, io,
    net::{TcpListener, TcpStream},
    path::PathBuf,
    process,
    sync::Arc,
    thread,
};

fn handle_connection(mut stream: TcpStream, base_path: Arc<PathBuf>) -> io::Result<usize> {
    let request = match Request::get(&mut stream) {
        Ok(request) => request,
        Err(e) => {
            return http_400(&mut stream, e);
        }
    };

    println!("{} {}", &request.method.cyan(), &request.path.yellow());

    if &request.method != "GET" {
        println!(
            "Requested Http Method: {} is not supported.",
            &request.method
        );
        return http_405(&mut stream);
    }

    let mut path = (*base_path).clone();
    if request.path != "/" {
        path.push(&request.path[1..]);
    }

    if !path.exists() {
        http_404(&mut stream, "Requested path does not exist.")
    } else if path.is_dir() {
        let index = path.join("index.html");
        if index.exists() {
            send_file(&mut stream, &index)
        } else {
            let base = base_path.to_str().unwrap();
            show_dir(&mut stream, base, path)
        }
    } else {
        send_file(&mut stream, path.as_path())
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
    let path = Arc::new(args.path);
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let path = path.clone();
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
