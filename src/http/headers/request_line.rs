use split::{buffer_split_mut, BufferExt};
use http::{RequestResult, RequestError, url_decode, UrlEncodedParams};
use std::ascii::AsciiExt;
use http::str::slice_to_str;

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
  pub version: &'a str,
  pub querystring: UrlEncodedParams<'a>
}

impl<'a> RequestLine<'a> {
  pub fn parse(line: &'a mut [u8]) -> RequestResult<RequestLine<'a>> {
    let mut words = buffer_split_mut(line, b" ").filter(|s| s.len() != 0);
    let method = words.next();
    let uri = words.next();
    let http_version = words.next();

    if let (Some(method), Some(uri), Some(http_version)) = (method, uri, http_version) {
      if let Some(version) = http_version.get(5..) {
        method.make_ascii_uppercase();

        let (uri, querystring) = if let Some(query_idx) = uri.position(b"?") {
          let (uri, querystring) = uri.split_at_mut(query_idx);
          let uri = url_decode(uri);
          let querystring = &mut querystring[1..];
          (uri, querystring)
        }
        else {
          let (querystring, uri) = uri.split_at_mut(0);
          let uri = url_decode(uri);
          (uri, querystring)
        };
        let uri = slice_to_str(uri)?;

        return Ok(RequestLine {
          method: slice_to_str(method)?,
          uri,
          querystring: UrlEncodedParams::decode_and_create(querystring)?,
          version: slice_to_str(version)?
        });
      }
    }
    Err(RequestError::InvalidRequestLine)
  }
}

#[cfg(test)]
mod tests {
  use test_helpers::copy_str;
  #[test]
  fn test_request_line() {
    let mut s = [0u8; 20];
    copy_str(&mut s, b"GET  /foo   HTTP/1.1");
    let req_line = super::RequestLine::parse(&mut s).unwrap();
    assert_eq!(req_line.method, "GET");
    assert_eq!(req_line.uri, "/foo");
    assert_eq!(req_line.version, "1.1");
  }
  #[test]
  fn test_request_line_lowercase_method() {
    let mut s = [0u8; 17];
    copy_str(&mut s, b"get /foo HTTP/1.1");
    let req_line = super::RequestLine::parse(&mut s).unwrap();
    assert_eq!(req_line.method, "GET");
    assert_eq!(req_line.uri, "/foo");
    assert_eq!(req_line.version, "1.1");
  }
  #[test]
  fn test_escaped_query() {
    let mut s = [0u8; 27];
    copy_str(&mut s, b"GET /foo%3F?%3Fbar HTTP/1.1");
    let req_line = super::RequestLine::parse(&mut s).unwrap();
    assert_eq!(req_line.uri, "/foo?");
    let qs_iter = req_line.querystring.iter();
    let bar = qs_iter.next().unwrap();
    assert_eq!(bar.name, "?bar");
  }
}