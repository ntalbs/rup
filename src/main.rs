use std::io::{self, ErrorKind, Read, Write, BufReader, BufRead};
use std::fs;
use std::net::{TcpListener, TcpStream};
use std::path::Path;
use std::thread;

/// Represents HTTP Request. Currently, only interested in `method` and `path`.
/// Though it has `method` field, only supported HTTP method will be GET, and
/// other methods in requests will cause error HTTP-404.
struct Request {
    method: String,
    path: String,
}

fn mime_type(path: &str) -> &'static str {
    match path.split('.').last() {
        Some(ext) => match ext {
            "html"|"htm" => "text/html",
            "txt" => "text/plain",
            "css" => "text/css",
            "js" => "application/javascript",
            "png" => "image/png",
            "jpg"|"jpeg" => "image/jpeg",
            "gif" => "image/gif",
            _ => "binary/octet-stream"
        },
        None => "binary/octet-stream"
    }
}

fn trim_path(input: &str) -> &str {
    input.split(|c| c == '#' || c == '?').next().unwrap()
}

fn decode_percent(s: &str) -> String {
    let mut decoded = String::new();
    let mut chars = s.chars();
    let mut buf: Vec<u8> = Vec::new();
    while let Some(ch) = chars.next() {
        if ch == '%' {
            let encoded = chars.next().unwrap().to_string() + &chars.next().unwrap().to_string();
            buf.push(u8::from_str_radix(&encoded, 16).unwrap());
        } else {
            if decoded.len() != 0 {
                decoded.push_str(&std::str::from_utf8(&buf).unwrap());
                buf.clear();
            }
            decoded.push(ch);
        }
    }
    decoded
}

impl TryFrom<String> for Request {
    type Error = &'static str;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        let v = s.split_whitespace().take(2).collect::<Vec<&str>>();
        if let [method, path] = &v[..] {
            Ok(Request { method: method.to_string(), path: format!(".{}", decode_percent(trim_path(path))) })
        } else {
            Err("Fail to get request method/path")
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

fn http_400(stream: &mut TcpStream) -> io::Result<u64> {
    let body = b"Bad Request\n";
    stream.write_all(b"HTTP/1.1 400 Bad Request\n").unwrap();
    stream.write_all(b"Content-Type: text/plain\n").unwrap();
    stream.write_all(format!("Content-Length: {}\r\n\r\n", body.len()).as_bytes()).unwrap();
    stream.write_all(body).unwrap();
    Err(io::Error::new(ErrorKind::Other, "400 Bad Request"))
}

fn http_404(stream: &mut TcpStream) -> io::Result<u64> {
    let body = b"Not Found\n";
    stream.write_all(b"HTTP/1.1 404 Not Fount\n").unwrap();
    stream.write_all(b"Content-Type: text/plain\n").unwrap();
    stream.write_all(format!("Content-Length: {}\r\n\r\n", body.len()).as_bytes()).unwrap();
    stream.write_all(body).unwrap();
    Err(io::Error::new(ErrorKind::Other, "404 Not Found"))
}

fn handle_connection(mut stream: TcpStream) -> io::Result<u64> {
    let mut request = match Request::try_from(request_line(&stream)) {
        Ok(request) => request,
        Err(_) => {
            eprintln!("Bad Request.");
            return http_400(&mut stream);
        }
    };

    println!("{} {}", &request.method, &request.path);

    if &request.method != "GET" {
        println!("Requested Http Method: {} is not supported.", &request.method);
        return http_404(&mut stream);
    }

    let file = fs::File::open(&request.path);
    return match file {
        Ok(mut f) => {
            let mut md = f.metadata().unwrap();
            if md.is_dir() {
                let index = Path::new(&request.path).join("index.html");
                if index.exists() {
                    f = fs::File::open(index).unwrap();
                    md = f.metadata().unwrap();
                    request.path = format!("{}/index.html", &request.path);
                } else {
                    panic!("Listing directory is not implemented yet...");
                }
            }
            stream.write_all(b"HTTP/1.1 200 OK\n").unwrap();
            stream.write_all(format!("Content-Type: {}\n", mime_type(&request.path)).as_bytes()).unwrap();
            stream.write_all(format!("Content-Length: {}\r\n\r\n", &md.len()).as_bytes()).unwrap();
            send(&mut f, &mut stream)
        }
        Err(_) => http_404(&mut stream),
    }
}

fn main() {
    let listener = TcpListener::bind("0.0.0.0:80").expect("Couldn't bind.");
    println!("Listening on :80\n");
    println!("Root: {:?}", Path::new(".").canonicalize().unwrap());
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
