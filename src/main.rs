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

fn trim_path(input: &str) -> &str {
    input.split(|c| c == '#' || c == '?').next().unwrap()
}

impl FromStr for Request {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let v = s.split_whitespace().take(2).collect::<Vec<&str>>();
        if let [method, path] = &v[..] {
            Ok(Request { method: method.to_string(), path: trim_path(path).to_string() })
        } else {
            Err("Fail to get request method/path")
        }
    }
}

impl Into<Request> for String {
    fn into(self) -> Request {
        let v = self.split_whitespace().take(2).collect::<Vec<&str>>();
        if let [method, path] = &v[..] {
            Request { method: method.to_string(), path: trim_path(path).to_string() }
        } else {
            panic!("Fail to get request method/path")
        }
    }
}

fn request_line(mut stream: &TcpStream) -> String {
    BufReader::new(&mut stream)
        .lines()
        .map(|result| result.unwrap())
        .take(1) // read only first line
        .next()
        .unwrap()
}

fn handle_connection(mut stream: TcpStream) -> Result<(), &'static str> {
    let request: Request = request_line(&stream).into();
    println!("{} {}", request.method, request.path);

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
