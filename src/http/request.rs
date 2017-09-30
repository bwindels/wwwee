use super::headers::*;

pub struct CommonHeaders<'a> {
  host: Option<&'a str>,
  content_length: Option<u64>,
  content_type: Option<MimeType>,
  authorization: Option<Authorization<'a>>
}

pub struct Request<'a> {
  request_line: RequestLine<'a>,
  headers: CommonHeaders<'a>,
  all_headers: HeadersIterator<'a>
}

impl<'a> Request<'a> {
  pub fn parse(header_bytes: &'a [u8]) -> ParseResult<Request<'a>> {

  }
}