use std::str::{from_utf8, Chars};

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

pub(crate) fn decode_percent(s: &str) -> Result<String, &'static str> {
    let mut decoded = String::new();
    let mut chars = s.chars();
    let mut buf: Vec<u8> = Vec::new();
    while let Some(ch) = chars.next() {
        if ch == '%' {
            let hex = get_hex(&mut chars)?;
            buf.push(hex);
        } else {
            if !decoded.is_empty() {
                decoded.push_str(from_utf8(&buf).unwrap());
                buf.clear();
            }
            decoded.push(ch);
        }
    }
    Ok(decoded)
}
