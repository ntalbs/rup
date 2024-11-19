use std::fs::{self, File};
use std::io::{self, BufRead, BufReader, ErrorKind, Read, Write};
use std::net::TcpStream;
use std::path::Path;

use colorust::Color;

use crate::decode::decode_percent;
use crate::mime::mime;

/// Represents HTTP Request. Currently, only interested in `method` and `path`.
/// Headers in requests are out of interest, hence will be ignored.
/// Though it has `method` field, only supported HTTP method will be GET, and
/// other methods in requests will cause error HTTP-405.
pub(crate) struct Request {
    pub method: String,
    pub path: String,
}

impl TryFrom<String> for Request {
    type Error = String;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        let v = s.split_whitespace().take(2).collect::<Vec<&str>>();
        if let [method, path] = &v[..] {
            let decoded = decode_percent(trim_path(path))?;
            Ok(Request {
                method: method.to_string(),
                path: decoded,
            })
        } else {
            Err(format!("Fail to get request method/path\n{}", s))
        }
    }
}

impl Request {
    pub fn get(stream: &mut TcpStream) -> Result<Self, String> {
        let mut line = String::new();
        if BufReader::new(stream).read_line(&mut line).is_ok() {
            Request::try_from(line)
        } else {
            Err("Fail to get request line".into())
        }
    }
}

fn mime_type(path: &Path) -> &'static str {
    mime(path.extension().and_then(|s| s.to_str()).unwrap_or(""))
}

fn trim_path(input: &str) -> &str {
    input.split(&['#', '?']).next().unwrap()
}

trait WriteFile {
    fn write_file(&mut self, file: fs::File) -> io::Result<usize>;
}

impl WriteFile for TcpStream {
    fn write_file(&mut self, mut file: File) -> io::Result<usize> {
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

pub(crate) enum Response<'a> {
    File(&'a Path),
    Directory(&'a str, &'a Path),
    Error { code: u16, body: &'a str },
}

impl<'a> Response<'a> {
    pub(crate) fn file(path: &'a Path) -> Self {
        Response::File(path)
    }

    pub(crate) fn directory(base: &'a str, path: &'a Path) -> Self {
        Response::Directory(base, path)
    }

    pub(crate) fn error(code: u16, body: &'a str) -> Self {
        Response::Error { code, body }
    }

    pub(crate) fn send_to(&self, stream: &mut TcpStream) -> io::Result<usize> {
        match *self {
            Response::File(path) => send_file(stream, path),
            Response::Directory(base, path) => show_dir(stream, base, path),
            Response::Error { code, body } => match code {
                400 => http_400(stream, body),
                404 => http_404(stream, body),
                405 => http_405(stream),
                _ => Err(io::Error::new(ErrorKind::Other, body)),
            },
        }
    }
}

pub(crate) fn send_file(stream: &mut TcpStream, path: &Path) -> io::Result<usize> {
    let f = File::open(path)?;
    let md = f.metadata()?;
    let mime_type = mime_type(path);

    stream.write_all(b"HTTP/1.1 200 OK\n")?;
    stream.write_all(b"Cache-Control: max-age=3600\n")?;
    if mime_type.contains("text") {
        stream.write_all(format!("Content-Type: {}; charset=utf-8\n", mime_type).as_bytes())?;
    } else {
        stream.write_all(format!("Content-Type: {}\n", mime_type).as_bytes())?;
    }
    stream.write_all(format!("Content-Length: {}\r\n\r\n", md.len()).as_bytes())?;
    stream.write_file(f)
}

fn css() -> &'static str {
    "<style>body { font-size: 1.2rem; line-height: 1.2; margin: 1rem; }</style>"
}

pub(crate) fn show_dir(stream: &mut TcpStream, base: &str, path: &Path) -> io::Result<usize> {
    let mut buf: Vec<u8> = Vec::new();
    buf.write_all(
        format!(
            "<html><head>{}</head><body><p style=\"color: #fff; background-color: #44f;\">Path: {}</p><ol>",
            css(),
            &path.to_str().unwrap()[1..]
        )
        .as_bytes(),
    )?;

    if base != path.to_str().unwrap() {
        buf.write_all("<li><a href=\"..\">..</a></li>".as_bytes())?;
    }

    let paths = fs::read_dir(path)?;
    for f in paths {
        let dir_entry = f?;
        if let (Ok(href), Some(name)) = (
            dir_entry.path().strip_prefix(base),
            dir_entry.path().file_name(),
        ) {
            let href = href.to_str().unwrap();
            let name = name.to_str().unwrap();
            buf.write_all(format!("<li><a href=\"/{}\">{}</li>", href, name).as_bytes())?;
        }
    }
    buf.write_all(b"</ol></body><html>")?;

    stream.write_all(b"HTTP/1.1 200 OK\n")?;
    stream.write_all(b"Content-Type: text/html; charset=utf-8\n")?;
    stream.write_all(format!("Content-Length: {}\r\n\r\n", buf.len()).as_bytes())?;
    stream.write_all(&buf)?;

    Ok(buf.len())
}

pub(crate) fn http_400(stream: &mut TcpStream, reason: &str) -> io::Result<usize> {
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

pub(crate) fn http_404(stream: &mut TcpStream, reason: &str) -> io::Result<usize> {
    stream.write_all(b"HTTP/1.1 404 Not Found\n")?;
    stream.write_all(b"Content-Type: text/plain\n")?;

    let path_404 = Path::new("./404.html");
    if path_404.exists() {
        let file_404 = File::open(path_404)?;
        let content_length = &file_404.metadata()?.len();
        stream.write_all(b"Content-Type: text/html\n")?;
        stream.write_all(format!("Content-Length: {}\r\n\r\n", content_length).as_bytes())?;
        stream.write_file(file_404)?;
    } else {
        let body_string = format!("Not Found: {}\n", reason);
        let body = body_string.as_bytes();
        stream.write_all(format!("Content-Length: {}\r\n\r\n", body.len()).as_bytes())?;
        stream.write_all(body)?;
    }

    Err(io::Error::new(
        ErrorKind::Other,
        format!("{}: {}", "404 Not Found".red(), reason),
    ))
}

pub(crate) fn http_405(stream: &mut TcpStream) -> io::Result<usize> {
    let body_string = "405 Method Not Allowed\n";
    let body = body_string.as_bytes();
    stream.write_all(b"HTTP/1.1 405 Method Not Allowed\n")?;
    stream.write_all(b"Allow: GET\n")?;
    stream.write_all(b"Content-Type: text/plain\n")?;
    stream.write_all(format!("Content-Length: {}\r\n\r\n", body.len()).as_bytes())?;
    stream.write_all(body)?;
    Err(io::Error::new(ErrorKind::Other, body_string))
}
