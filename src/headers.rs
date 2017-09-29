use std::ascii::AsciiExt;
use std::cmp;
use str::str_split_mut;
use error::{ParseResult, ParseError};

pub struct RequestLine<'a> {
  pub method: &'a str,
  pub uri: &'a str,
  pub version: &'a str
}

impl<'a> RequestLine<'a> {
  pub fn parse(line: &'a mut str) -> ParseResult<RequestLine<'a>> {
    let mut words = str_split_mut(line, " ").filter(|s| s.len() != 0);
    let method = words.next();
    let uri = words.next();
    let http_version = words.next();

    if let (Some(method), Some(uri), Some(http_version)) = (method, uri, http_version) {
      if let Some(version) = http_version.get(5..) {
        method.make_ascii_uppercase();
        return Ok(RequestLine {
          method,
          uri,
          version
        });
      }
    }
    Err(ParseError::InvalidRequestLine)
  }
}

pub struct Header<'a> {
  pub name: &'a str,
  pub value: &'a str
}

impl<'a> Header<'a> {
  pub fn parse(line: &'a mut str) -> ParseResult<Header<'a>> {
    if let Some(idx) = line.find(":") {
      let (name, value) = line.split_at_mut(idx);
      name.make_ascii_uppercase();
      for name_word in str_split_mut(name, "-") {
        name_word[0..1].make_ascii_uppercase();
        name_word[1..].make_ascii_lowercase();
      }
      let name = name.trim();
      let value = &value[1..];  //cut : off
      let value = value.trim();
      Ok(Header{name, value})
    }
    else {
      Err(ParseError::InvalidHeader)
    }
  }
}

pub struct Headers<'a> {
  buffer: &'a mut str
}

pub struct HeaderBodySplitter {
  find_offset: usize
}

impl HeaderBodySplitter {
  pub fn new() -> HeaderBodySplitter {
    HeaderBodySplitter{find_offset: 1}
  }

  pub fn update<'a>(&mut self, buffer: &'a mut [u8]) -> Option<(&'a mut [u8], &'a mut [u8])> {
    const HEADER_END: &'static [u8] = b"\r\n\r\n";
    let offset = cmp::max(HEADER_END.len(), self.find_offset + 1) - HEADER_END.len();
    //update the offset where to look from next update
    self.find_offset = buffer.len();

    buffer.get(offset..).and_then(|search_space| {
      search_space.windows(HEADER_END.len())
        .position(|window| window == HEADER_END)
    })
    .map(|header_end| offset + header_end + HEADER_END.len())
    .map(move |header_end| buffer.split_at_mut(header_end))
    .map(|(headers, body)| {
      let len = headers.len();
      (&mut headers[..len - HEADER_END.len()], body)
    })
  }
}

#[cfg(test)]
mod tests {
  #[test]
  fn test_request_line() {
    let mut s = "GET  /foo   HTTP/1.1".to_string();
    let req_line = super::RequestLine::parse(s.as_mut_str()).unwrap();
    assert_eq!(req_line.method, "GET");
    assert_eq!(req_line.uri, "/foo");
    assert_eq!(req_line.version, "1.1");
  }
  #[test]
  fn test_request_line_lowercase_method() {
    let mut s = "get /foo HTTP/1.1".to_string();
    let req_line = super::RequestLine::parse(s.as_mut_str()).unwrap();
    assert_eq!(req_line.method, "GET");
    assert_eq!(req_line.uri, "/foo");
    assert_eq!(req_line.version, "1.1");
  }
  #[test]
  fn test_parse_header() {
    let mut s = "aCCEPT : text/plain".to_string();
    let header = super::Header::parse(s.as_mut_str()).unwrap();
    assert_eq!(header.name, "Accept");
    assert_eq!(header.value, "text/plain");
  }
  #[test]
  fn test_parse_header_multi_word() {
    let mut s = "CONTENT-type : text/plain".to_string();
    let header = super::Header::parse(s.as_mut_str()).unwrap();
    assert_eq!(header.name, "Content-Type");
    assert_eq!(header.value, "text/plain");
  }

  #[test]
  fn test_header_body_splitter() {
    let mut st = "foobar\r\nhello\r\n\r\nhaha".to_string();
    let mut s = unsafe { st.as_bytes_mut() };
    let mut splitter = super::HeaderBodySplitter::new();
    assert_eq!(splitter.update(&mut s[0..13]), None);
    assert_eq!(splitter.update(&mut s[0..16]), None);
    match splitter.update(&mut s) {
      Some((headers, body)) => {
        assert_eq!(headers, b"foobar\r\nhello");
        assert_eq!(body, b"haha");
      }
      None => panic!("should be Some")
    };
  }

}