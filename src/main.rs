use std::error::Error;
use std::io::{Write, BufReader, BufRead};
use std::net::{TcpListener, TcpStream};
use std::thread;

fn get_method_and_path(line: String) -> (String, String) {
    let v = line.split_whitespace().take(2).collect::<Vec<&str>>();
    (v[0].to_string(), v[1].to_string())
}

fn handle_connection(mut stream: TcpStream) {
    let http_request = BufReader::new(&mut stream)
        .lines()
        .map(|result| result.unwrap())
        .take_while(|line| !line.is_empty())
        .next()
        .unwrap();

    let (method, path) = get_method_and_path(http_request);
    println!("Method: {method}");
    println!("Path: {path}");

    let body = "Hello, world!\n";

    stream.write(b"HTTP/1.1 200 OK\n").unwrap();
    stream.write(b"Content-Length: 14\n").unwrap();
    stream.write(b"Content-Type: text/plain\n\n").unwrap();
    stream.write(body.as_bytes()).unwrap();
}

fn main() {
    let listener = TcpListener::bind("0.0.0.0:80").expect("Couldn't bind.");
    println!("Listening on :80\n");
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                thread::spawn(move || {
                    handle_connection(stream);
                });
            }
            Err(e) => {
                eprintln!("failed: {e}");
            }
        }
    }
}
