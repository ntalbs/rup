use std::io::{self, ErrorKind, Read, Write, BufReader, BufRead};
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
            Request {
                method: method.to_string(),
                path: format!("./{}", trim_path(path).to_string())
            }
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

const BUF_SIZE: usize = 8 * 1024;
fn send<R: ?Sized, W: ?Sized>(reader: &mut R, writer: &mut W) -> io::Result<u64> where R: Read, W: Write {
    let mut buf = [0; BUF_SIZE];
    let mut written = 0;
    loop {
        let len = match reader.read(&mut buf) {
            Ok(0) => return Ok(written),
            Ok(len) => len,
            Err(ref e) if e.kind() == ErrorKind::Interrupted => continue,
            Err(e) => return Err(e),
        };
        writer.write_all(&buf[..len])?;
        written += len as u64;
    }
}

fn http_404(stream: &mut TcpStream) -> io::Result<u64> {
    let body = b"Not Found\n";
    stream.write(b"HTTP/1.1 404 Not Fount\n").unwrap();
    stream.write(b"Content-Type: text/plain\n").unwrap();
    stream.write(format!("Content-Length: {}\r\n\r\n", body.len()).as_bytes()).unwrap();
    stream.write(body).unwrap();
    Err(io::Error::new(ErrorKind::Other, "404 Not Fount"))
}

fn handle_connection(mut stream: TcpStream) -> io::Result<u64> {
    let request: Request = request_line(&stream).into();
    println!("{} {}", request.method, request.path);

    let file = std::fs::File::open(&request.path);
    return match file {
        Ok(mut f) => {
            stream.write(b"HTTP/1.1 200 OK\n").unwrap();
            stream.write(b"Content-Type: text/plain\n").unwrap();
            stream.write(format!("Content-Length: {}\r\n\r\n", f.metadata().unwrap().len()).as_bytes()).unwrap();
            send(&mut f, &mut stream)
        }
        Err(_) => http_404(&mut stream),
    }
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
