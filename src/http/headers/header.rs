use super::{MimeType, Authorization, ContentRange, RawHeader};
use http::{RequestResult, RequestError};
use http::str::slice_to_str;
use std::str::FromStr;

pub enum Header<'a> {
  Host(&'a str),
  ContentLength(u64),
  ContentType(MimeType<'a>),
  Authorization(Authorization<'a>),
  Referer(&'a str),
  Range(ContentRange<'a>),
  Other(RawHeader<'a>)
}

fn parse_u64(num_str: &str) -> RequestResult<u64> {
  u64::from_str(num_str).map_err(|_| RequestError::InvalidHeader)
}

impl<'a> Header<'a> {
  pub fn from_raw(raw_header: RawHeader<'a>) -> RequestResult<Header<'a>> {
    let value = slice_to_str(raw_header.value)?;
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

