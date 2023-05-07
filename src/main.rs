pub(crate) mod colored;

use std::fs;
use std::io::{self, BufRead, BufReader, ErrorKind, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::Path;
use std::str::Chars;
use std::thread;

use crate::colored::Colorize;

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
            "html" | "htm" => "text/html",
            "txt" => "text/plain",
            "css" => "text/css",
            "js" => "application/javascript",
            "png" => "image/png",
            "jpg" | "jpeg" => "image/jpeg",
            "gif" => "image/gif",
            _ => "binary/octet-stream",
        },
        None => "binary/octet-stream",
    }
}

fn trim_path(input: &str) -> &str {
    input.split(|c| c == '#' || c == '?').next().unwrap()
}

fn get_hex(chars: &mut Chars) -> Result<u8, &'static str> {
    const MALFORMED_URI: &str = "Malformed URI";
    let digit1 = match chars.next() {
        Some(c) => c,
        None => return Err(MALFORMED_URI),
    };
    let digit2 = match chars.next() {
        Some(c) => c,
        None => return Err(MALFORMED_URI),
    };
    let encoded = format!("{digit1}{digit2}");
    match u8::from_str_radix(&encoded, 16) {
        Ok(xx) => Ok(xx),
        Err(_) => Err(MALFORMED_URI),
    }
}

fn decode_percent(s: &str) -> Result<String, &'static str> {
    let mut decoded = String::new();
    let mut chars = s.chars();
    let mut buf: Vec<u8> = Vec::new();
    while let Some(ch) = chars.next() {
        if ch == '%' {
            let hex = get_hex(&mut chars)?;
            buf.push(hex);
        } else {
            if !decoded.is_empty() {
                decoded.push_str(std::str::from_utf8(&buf).unwrap());
                buf.clear();
            }
            decoded.push(ch);
        }
    }
    Ok(decoded)
}

impl TryFrom<String> for Request {
    type Error = &'static str;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        let v = s.split_whitespace().take(2).collect::<Vec<&str>>();
        if let [method, path] = &v[..] {
            let decoded = decode_percent(trim_path(path))?;
            Ok(Request {
                method: method.to_string(),
                path: format!(".{}", decoded),
            })
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
fn send_file(stream: &mut TcpStream, path: &Path) -> io::Result<u64> {
    let mut f = fs::File::open(path)?;
    let md = f.metadata()?;

    stream.write_all(b"HTTP/1.1 200 OK\n").unwrap();
    stream
        .write_all(format!("Content-Type: {}\n", mime_type(path.to_str().unwrap())).as_bytes())
        .unwrap();
    stream
        .write_all(format!("Content-Length: {}\r\n\r\n", &md.len()).as_bytes())
        .unwrap();

    let mut buf = [0; BUF_SIZE];
    let mut written = 0;
    loop {
        let len = match f.read(&mut buf) {
            Ok(0) => return Ok(written),
            Ok(len) => len,
            Err(ref e) if e.kind() == ErrorKind::Interrupted => continue,
            Err(e) => return Err(e),
        };
        stream.write_all(&buf[..len])?;
        written += len as u64;
    }
}

fn show_dir(stream: &mut TcpStream, path: &Path) -> io::Result<u64> {
    let mut buf: Vec<u8> = Vec::new();
    buf.write_all(b"<html><body><ol>")?;

    if let Some(parent) = path.parent() {
        match parent.to_str().unwrap() {
            "" => {}
            "." => buf.write_all(b"<li><a href=\"/\">..</a></li>")?,
            _ => buf.write_all(
                format!(
                    "<li><a href=\"{}\">..</a></li>",
                    &parent.to_str().unwrap()[1..]
                )
                .as_bytes(),
            )?,
        }
    }

    let paths = fs::read_dir(path)?;
    for f in paths {
        let dir_entry = f?;
        let href = dir_entry.path();
        buf.write_all(
            format!(
                "<li><a href=\"{}\">{}</a></li>",
                &href.to_str().unwrap()[1..],
                dir_entry.path().display()
            )
            .as_bytes(),
        )?;
    }
    buf.write_all(b"</ol></body><html>")?;

    stream.write_all(b"HTTP/1.1 200 OK\n")?;
    stream.write_all(b"Content-Type: text/html\n")?;
    stream.write_all(format!("Content-Length: {}\r\n\r\n", buf.len()).as_bytes())?;
    stream.write_all(&buf)?;

    Ok(buf.len() as u64)
}

fn http_400(stream: &mut TcpStream, reason: &str) -> io::Result<u64> {
    let body_string = format!("Bad Request: {}\n", reason);
    let body = body_string.as_bytes();
    stream.write_all(b"HTTP/1.1 400 Bad Request\n")?;
    stream.write_all(b"Content-Type: text/plain\n")?;
    stream.write_all(format!("Content-Length: {}\r\n\r\n", body.len()).as_bytes())?;
    stream.write_all(body)?;
    Err(io::Error::new(
        ErrorKind::Other,
        format!("{}: {}", "400 Bad Request".red(), reason),
    ))
}

fn http_404(stream: &mut TcpStream, reason: &str) -> io::Result<u64> {
    let body_string = format!("Not Found: {}\n", reason);
    let body = body_string.as_bytes();
    stream.write_all(b"HTTP/1.1 404 Not Fount\n")?;
    stream.write_all(b"Content-Type: text/plain\n")?;
    stream.write_all(format!("Content-Length: {}\r\n\r\n", body.len()).as_bytes())?;
    stream.write_all(body)?;
    Err(io::Error::new(
        ErrorKind::Other,
        format!("{}: {}", "404 Not Found".red(), reason),
    ))
}

fn handle_connection(mut stream: TcpStream) -> io::Result<u64> {
    let request = match Request::try_from(request_line(&stream)) {
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
        return http_404(
            &mut stream,
            &format!(
                "Requested Http Method: {} is not supported.",
                &request.method
            ),
        );
    }

    let path = Path::new(&request.path);
    if !path.exists() {
        http_404(&mut stream, "Requested path does not exist.")
    } else if path.is_dir() {
        let index = path.join("index.html");
        if index.exists() {
            send_file(&mut stream, &index)
        } else {
            show_dir(&mut stream, path)
        }
    } else {
        send_file(&mut stream, path)
    }
}

const VERSION: &str = env!("CARGO_PKG_VERSION");
fn main() {
    println!("{} {}", "Rup version:".yellow(), VERSION.green());
    println!(
        "{} {}",
        "Starting server".yellow(),
        "on http://localhost".green()
    );
    let listener = TcpListener::bind("0.0.0.0:80").expect("Couldn't bind.");
    println!(
        "{} {}",
        "Serving ".yellow(),
        Path::new(".")
            .canonicalize()
            .unwrap()
            .to_str()
            .unwrap()
            .green()
    );
    println!("Hit Ctrl+C to exit.");
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                thread::spawn(move || match handle_connection(stream) {
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
