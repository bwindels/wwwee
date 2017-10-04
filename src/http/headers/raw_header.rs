use std::ascii::AsciiExt;
use http::{RequestResult, RequestError};
use str::str_split_mut;

pub struct RawHeader<'a> {
  pub name: &'a str,
  pub value: &'a str
}

impl<'a> RawHeader<'a> {
  pub fn parse(line: &'a mut str) -> RequestResult<RawHeader<'a>> {
    if let Some(idx) = line.find(":") {
      let (name, value) = line.split_at_mut(idx);
      name.make_ascii_uppercase();
      for name_word in str_split_mut(name, "-") {
        name_word[0..1].make_ascii_uppercase();
        name_word[1..].make_ascii_lowercase();
      }
      let name = name.trim();
      let value = &value[1..];  //cut ':' off
      let value = value.trim();
      Ok(RawHeader{name, value})
    }
    else {
      Err(RequestError::InvalidHeader)
    }
  }
}


#[cfg(test)]
mod tests {
  #[test]
  fn test_parse_header() {
    let mut s = "aCCEPT : text/plain".to_string();
    let header = super::RawHeader::parse(s.as_mut_str()).unwrap();
    assert_eq!(header.name, "Accept");
    assert_eq!(header.value, "text/plain");
  }
  #[test]
  fn test_parse_header_multi_word() {
    let mut s = "CONTENT-type : text/plain".to_string();
    let header = super::RawHeader::parse(s.as_mut_str()).unwrap();
    assert_eq!(header.name, "Content-Type");
    assert_eq!(header.value, "text/plain");
  }
}