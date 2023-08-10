mod cli;
mod color;
mod decode;
mod mime;

use std::io::{self, BufRead, BufReader, ErrorKind, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::Path;
use std::{env, thread};
use std::{fs, str};

use crate::cli::Args;
use crate::color::Color;
use crate::decode::decode_percent;
use crate::mime::mime;

/// Represents HTTP Request. Currently, only interested in `method` and `path`.
/// Though it has `method` field, only supported HTTP method will be GET, and
/// other methods in requests will cause error HTTP-404.
struct Request {
    method: String,
    path: String,
}

fn mime_type(path: &Path) -> &'static str {
    mime(path.extension().and_then(|s| s.to_str()).unwrap_or(""))
}

fn trim_path(input: &str) -> &str {
    input.split(|c| c == '#' || c == '?').next().unwrap()
}

impl TryFrom<String> for Request {
    type Error = &'static str;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        let v = s.split_whitespace().take(2).collect::<Vec<&str>>();
        if let [method, path] = &v[..] {
            let decoded = decode_percent(trim_path(path))?;
            Ok(Request {
                method: method.to_string(),
                path: decoded,
            })
        } else {
            Err("Fail to get request method/path")
        }
    }
}

fn request_line(mut stream: &TcpStream) -> io::Result<String> {
    let mut buf = String::new();
    BufReader::new(&mut stream).read_line(&mut buf)?;
    Ok(buf)
}

trait WriteFile {
    fn write_file(&mut self, file: fs::File) -> io::Result<usize>;
}

impl WriteFile for TcpStream {
    fn write_file(&mut self, mut file: fs::File) -> io::Result<usize> {
        const BUF_SIZE: usize = 8 * 1024;
        let mut buf = [0; BUF_SIZE];
        let mut written = 0;
        loop {
            let len = match file.read(&mut buf) {
                Ok(0) => return Ok(written),
                Ok(len) => len,
                Err(ref e) if e.kind() == ErrorKind::Interrupted => continue,
                Err(e) => return Err(e),
            };
            self.write_all(&buf[..len])?;
            written += len;
        }
    }
}

fn send_file(stream: &mut TcpStream, path: &Path) -> io::Result<usize> {
    let f = fs::File::open(path)?;
    let md = f.metadata()?;
    let mime_type = mime_type(path);

    stream.write_all(b"HTTP/1.1 200 OK\n")?;
    stream.write_all(b"Cache-Control: max-age=3600\n")?;
    stream.write_all(format!("Content-Type: {}\n", mime_type).as_bytes())?;
    stream.write_all(format!("Content-Length: {}\r\n\r\n", &md.len()).as_bytes())?;
    stream.write_file(f)
}

fn show_dir(stream: &mut TcpStream, path: &Path) -> io::Result<usize> {
    let mut buf: Vec<u8> = Vec::new();
    buf.write_all(
        format!(
            "<html><body><p style=\"color: #fff; background-color: #44f;\">Path: {}</p><ol>",
            &path.to_str().unwrap()[1..]
        )
        .as_bytes(),
    )?;

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
                dir_entry.path().file_name().unwrap().to_str().unwrap()
            )
            .as_bytes(),
        )?;
    }
    buf.write_all(b"</ol></body><html>")?;

    stream.write_all(b"HTTP/1.1 200 OK\n")?;
    stream.write_all(b"Content-Type: text/html\n")?;
    stream.write_all(format!("Content-Length: {}\r\n\r\n", buf.len()).as_bytes())?;
    stream.write_all(&buf)?;

    Ok(buf.len())
}

fn http_400(stream: &mut TcpStream, reason: &str) -> io::Result<usize> {
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

fn http_404(stream: &mut TcpStream, reason: &str) -> io::Result<usize> {
    let body_string = format!("Not Found: {}\n", reason);
    let body = body_string.as_bytes();
    stream.write_all(b"HTTP/1.1 404 Not Found\n")?;
    stream.write_all(b"Content-Type: text/plain\n")?;
    stream.write_all(format!("Content-Length: {}\r\n\r\n", body.len()).as_bytes())?;
    stream.write_all(body)?;
    Err(io::Error::new(
        ErrorKind::Other,
        format!("{}: {}", "404 Not Found".red(), reason),
    ))
}

fn http_405(stream: &mut TcpStream) -> io::Result<usize> {
    let body_string = "405 Method Not Allowed\n";
    let body = body_string.as_bytes();
    stream.write_all(b"HTTP/1.1 405 Method Not Allowed\n")?;
    stream.write_all(b"Allow: GET\n")?;
    stream.write_all(b"Content-Type: text/plain\n")?;
    stream.write_all(format!("Content-Length: {}\r\n\r\n", body.len()).as_bytes())?;
    stream.write_all(body)?;
    Err(io::Error::new(
        ErrorKind::Other,
        body_string,
    ))
}

fn handle_connection(mut stream: TcpStream) -> io::Result<usize> {
    let request_line = match request_line(&stream) {
        Ok(req_line) => req_line,
        Err(e) => {
            return http_400(&mut stream, &e.to_string());
        }
    };
    let request = match Request::try_from(request_line) {
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

    let temp = format!(".{}", &request.path);
    let path = Path::new(&temp);

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

    let listener = TcpListener::bind(format!("0.0.0.0:{port}")).expect("Couldn't bind.");
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
    println!("Hit Ctrl+C to exit.\n");
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
