use std::ascii::AsciiExt;
use http::{RequestResult, RequestError};
use http::str::*;
use split::{buffer_split_mut, BufferExt};
use std::str;

pub struct RawHeader<'a> {
  pub name: &'a str,
  pub value: &'a [u8]
}

impl<'a> RawHeader<'a> {
  pub fn parse(line: &'a mut [u8]) -> RequestResult<RawHeader<'a>> {
    if let Some(idx) = line.find(b":") {
      let (name, value) = line.split_at_mut(idx);
      name.make_ascii_uppercase();
      for name_word in buffer_split_mut(name, b"-") {
        name_word[0..1].make_ascii_uppercase();
        name_word[1..].make_ascii_lowercase();
      }
      let name = trim(name, is_whitespace);
      let name = slice_to_str(name)?;
      let value = &value[1..];  //cut ':' off
      let value = trim(value, is_whitespace);
      Ok(RawHeader{name, value})
    }
    else {
      Err(RequestError::InvalidHeader)
    }
  }
}


#[cfg(test)]
mod tests {

  use test_helpers::copy_str;

  #[test]
  fn test_parse_header() {
    let mut s = [0u8; 19];
    copy_str(&mut s, b"aCCEPT : text/plain");
    let header = super::RawHeader::parse(&mut s).unwrap();
    assert_eq!(header.name, "Accept");
    assert_eq!(header.value, b"text/plain");
  }
  #[test]
  fn test_parse_header_multi_word() {
    let mut s = [0u8; 25];
    copy_str(&mut s, b"CONTENT-type : text/plain");
    let header = super::RawHeader::parse(&mut s).unwrap();
    assert_eq!(header.name, "Content-Type");
    assert_eq!(header.value, b"text/plain");
  }
}