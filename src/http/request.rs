use super::headers::*;
use super::{RequestResult, RequestError};
use split::buffer_split_mut;
use std::str;

pub struct CommonHeaders<'a> {
  pub host: Option<&'a str>,
  pub referer: Option<&'a str>,
  pub content_length: Option<u64>,
  pub content_type: Option<MimeType<'a>>,
  pub authorization: Option<Authorization<'a>>
}

impl<'a> CommonHeaders<'a> {
  pub fn new() -> CommonHeaders<'a> {
    CommonHeaders {
      host: None,
      referer: None,
      content_length: None,
      content_type: None,
      authorization: None,
    }
  }

  pub fn set_header(&mut self, header: Header<'a>) {
    match header {
      Header::Host(host) => self.host = Some(host),
      Header::ContentLength(len) => self.content_length = Some(len),
      Header::ContentType(t) => self.content_type = Some(t),
      Header::Authorization(a) => self.authorization = Some(a),
      Header::Referer(r) => self.referer = Some(r),
      _ => ()
    };
  }
}

pub struct Request<'a> {
  request_line: RequestLine<'a>,
  headers: CommonHeaders<'a>
}

impl<'a> Request<'a> {
  pub fn parse(header_bytes: &'a mut [u8]) -> RequestResult<Request<'a>> {
    let mut headers = CommonHeaders::new();
    let mut request_line: Option<RequestLine> = None;

    for line in buffer_split_mut(header_bytes, b"\r\n") {
      if request_line.is_none() {
        request_line = Some(RequestLine::parse(line)?);
      }
      else {
        let raw_header = RawHeader::parse(line)?;
        let header = Header::from_raw(raw_header)?;
        headers.set_header(header);
      }
    }

    if let Some(request_line) = request_line {
      Ok(Request {
        request_line,
        headers
      })
    }
    else {
      Err(RequestError::InvalidRequestLine)
    }
  }

  pub fn uri(&self) -> &'a str {
    self.request_line.uri
  }

  pub fn method(&self) -> &'a str {
    self.request_line.method
  }

  pub fn querystring(&self) -> &'a str {
    self.request_line.querystring
  }

  pub fn headers(&self) -> &'a CommonHeaders {
    &self.headers
  }
  
}