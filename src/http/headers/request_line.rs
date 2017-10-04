use str::str_split_mut;
use http::{RequestResult, RequestError};
use std::ascii::AsciiExt;

pub struct HttpVersion {
  
}

impl HttpVersion {
  fn parse(version: &str) -> RequestResult<HttpVersion> {
    Err(RequestError::InvalidHeader)
  }
}

pub struct RequestLine<'a> {
  pub method: &'a str,
  pub uri: &'a str,
  pub version: &'a str
}

impl<'a> RequestLine<'a> {
  pub fn parse(line: &'a mut str) -> RequestResult<RequestLine<'a>> {
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
    Err(RequestError::InvalidRequestLine)
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
}