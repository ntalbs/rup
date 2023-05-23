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
    loop {
        match chars.next() {
            Some('%') => {
                let hex = get_hex(&mut chars)?;
                buf.push(hex);
            },
            Some(ch) => {
                if !buf.is_empty() {
                    decoded.push_str(from_utf8(&buf).unwrap());
                    buf.clear();
                }
                decoded.push(ch);
            },
            None => {
                if !buf.is_empty() {
                    decoded.push_str(from_utf8(&buf).unwrap());
                    buf.clear();
                }
                break;
            },
        }
    }
    Ok(decoded)
}

#[test]
fn test_decode() -> Result<(), &'static str> {
    assert_eq!(decode_percent("hello%20world")?, "hello world");
    assert_eq!(decode_percent("%ec%95%84%eb%a7%88%ec%a1%b4")?, "아마존");
    assert_eq!(decode_percent("/%ec%95%84%eb%a7%88%ec%a1%b4")?, "/아마존");
    assert_eq!(decode_percent("%ec%95%84%eb%a7%88%ec%a1%b4/")?, "아마존/");
    assert_eq!(decode_percent("/%ec%95%84%eb%a7%88%ec%a1%b4/")?, "/아마존/");
    Ok(())
}
