use super::{MimeType, Authorization, ContentRange, RawHeader};
use error::{ParseResult, ParseError};

pub enum Header<'a> {
  Host(&'a str),
  ContentLength(u64),
  ContentType(MimeType),
  Authorization(Authorization<'a>),
  Referer(&'a str),
  Range(ContentRange),
  Other(RawHeader<'a>)
}

fn parse_u64(num_str: &str) -> ParseResult<u64> {
  Err(ParseError::InvalidHeader)
}

impl<'a> Header<'a> {
  fn from_raw(raw_header: RawHeader<'a>) -> ParseResult<Header<'a>> {
    let value = raw_header.value;
    let header = match raw_header.name {
      "Host" => Header::Host(value),
      "Authorization" => Header::Authorization(Authorization::parse(value)?),
      "Referer" => Header::Referer(value),
      "Content-Type" => Header::ContentType(MimeType::parse(value)?),
      "Content-Length" => Header::ContentLength(parse_u64(value)?),
      "Range" => Header::Range(ContentRange::parse(value)?),
      _ => Header::Other(raw_header)
    };
    Ok(header)
  }
}

