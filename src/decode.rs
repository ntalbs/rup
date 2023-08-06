use std::str::{from_utf8, Chars};

const MALFORMED_URI: &str = "Malformed URI";
fn get_hex(chars: &mut Chars) -> Result<u8, &'static str> {
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

pub(crate) fn decode_percent(input: &str) -> Result<String, &'static str> {
    fn flush_buf(buf: &mut Vec<u8>, dest: &mut String) -> Result <(), &'static str> {
        if !buf.is_empty() {
            let ch = match from_utf8(buf) {
                Ok(s) => s,
                Err(_) => return Err(MALFORMED_URI),
            };
            dest.push_str(ch);
            buf.clear();
        }
        Ok(())
    }

    let mut decoded = String::new();
    let mut chars = input.chars();
    let mut buf: Vec<u8> = Vec::new();
    loop {
        match chars.next() {
            Some('%') => {
                let hex = get_hex(&mut chars)?;
                buf.push(hex);
            }
            Some(ch) => {
                flush_buf(&mut buf, &mut decoded)?;
                decoded.push(ch);
            }
            None => {
                flush_buf(&mut buf, &mut decoded)?;
                break;
            }
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

#[test]
fn test_invalid() {
    assert_eq!(decode_percent("%hello").unwrap_err(), MALFORMED_URI);
    assert_eq!(decode_percent("%1%1%3").unwrap_err(), MALFORMED_URI);
    assert_eq!(decode_percent("%ff").unwrap_err(), MALFORMED_URI);
}
