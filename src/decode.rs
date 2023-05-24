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
    fn flush_buf(buf: &mut Vec<u8>, dest: &mut String) {
        if !buf.is_empty() {
            dest.push_str(from_utf8(buf).unwrap());
            buf.clear();
        }
    }

    let mut decoded = String::new();
    let mut chars = s.chars();
    let mut buf: Vec<u8> = Vec::new();
    loop {
        match chars.next() {
            Some('%') => {
                let hex = get_hex(&mut chars)?;
                buf.push(hex);
            }
            Some(ch) => {
                flush_buf(&mut buf, &mut decoded);
                decoded.push(ch);
            }
            None => {
                flush_buf(&mut buf, &mut decoded);
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
