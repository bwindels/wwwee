use split::{buffer_split_mut};
use http::{RequestResult, RequestError, url_decode, UrlEncodedParams};
use http::str::{
  slice_to_str,
  try_split_two_mut
};

pub struct HttpVersion {
  
}

impl HttpVersion {
  fn parse(_version: &str) -> RequestResult<HttpVersion> {
    Err(RequestError::InvalidHeader)
  }
}

pub struct RequestLine<'a> {
  pub method: &'a str,
  pub url: &'a str,
  pub version: &'a str,
  pub query_params: UrlEncodedParams<'a>
}

impl<'a> RequestLine<'a> {
  pub fn parse(line: &'a mut [u8]) -> RequestResult<RequestLine<'a>> {
    let mut words = buffer_split_mut(line, b" ").filter(|s| s.len() != 0);
    let method = words.next();
    let url = words.next();
    let http_version = words.next();

    if let (Some(method), Some(url), Some(http_version)) = (method, url, http_version) {
      if let Some(version) = http_version.get(5..) {
        method.make_ascii_uppercase();
        let (url, querystring) = try_split_two_mut(url, b"?");
        let querystring = querystring.unwrap_or([0u8; 0].as_mut());
        let url = url_decode(url);

        return Ok(RequestLine {
          method: slice_to_str(method)?,
          url: slice_to_str(url)?,
          query_params: UrlEncodedParams::decode_and_create(querystring)?,
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
    assert_eq!(req_line.url, "/foo");
    assert_eq!(req_line.version, "1.1");
  }
  #[test]
  fn test_request_line_lowercase_method() {
    let mut s = [0u8; 17];
    copy_str(&mut s, b"get /foo HTTP/1.1");
    let req_line = super::RequestLine::parse(&mut s).unwrap();
    assert_eq!(req_line.method, "GET");
    assert_eq!(req_line.url, "/foo");
    assert_eq!(req_line.version, "1.1");
  }
  #[test]
  fn test_escaped_query() {
    let mut s = [0u8; 27];
    copy_str(&mut s, b"GET /foo%3F?%3Fbar HTTP/1.1");
    let req_line = super::RequestLine::parse(&mut s).unwrap();
    assert_eq!(req_line.url, "/foo?");
    let mut qs_iter = req_line.query_params.iter();
    let bar = qs_iter.next().unwrap();
    assert_eq!(bar.name, "?bar");
  }
}
