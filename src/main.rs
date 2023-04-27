use std::io::{Write, BufReader, BufRead};
use std::net::{TcpListener, TcpStream};
use std::str::FromStr;
use std::thread;

/// Represents HTTP Request. Currently, only interested in `method` and `path`.
/// Though it has `method` field, only supported HTTP method will be GET, and
/// other methods in requests will cause error HTTP-404.
struct Request {
    method: String,
    path: String,
}

impl FromStr for Request {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let v = s.split_whitespace().take(2).collect::<Vec<&str>>();
        if let [method, path] = &v[..] {
            Ok(Request { method: method.to_string(), path: path.to_string() })
        } else {
            Err("Fail to get request method/path")
        }
    }
}

fn handle_connection(mut stream: TcpStream) -> Result<(), &'static str> {
    let request_line = BufReader::new(&mut stream)
        .lines()
        .take(1) // read only first line
        .map(|result| result.unwrap())
        .next()
        .unwrap();

    let request = Request::from_str(&request_line)?;
    println!("{} => {} {}", request_line, request.method, request.path);

    let body = "Hello, world!\n";

    stream.write(b"HTTP/1.1 200 OK\n").unwrap();
    stream.write(b"Content-Length: 14\n").unwrap();
    stream.write(b"Content-Type: text/plain\n\n").unwrap();
    stream.write(body.as_bytes()).unwrap();

    Ok(())
}

fn main() {
    let listener = TcpListener::bind("0.0.0.0:80").expect("Couldn't bind.");
    println!("Listening on :80\n");
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                thread::spawn(move || {
                    match handle_connection(stream) {
                        Ok(_) => {},
                        Err(e) => eprintln!("{e:?}")
                    }
                });
            }
            Err(e) => {
                eprintln!("failed: {e}");
            }
        }
    }
}
