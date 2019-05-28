use super::{MimeType, Authorization, ContentRange, RawHeader, ETagMatch};
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
  Other(RawHeader<'a>),
  IfNoneMatch(ETagMatch<'a>),
}

fn parse_u64(num_str: &str) -> RequestResult<u64> {
  u64::from_str(num_str).map_err(|_| RequestError::InvalidHeader)
}

impl<'a> Header<'a> {
  pub fn from_raw(raw_header: RawHeader<'a>) -> RequestResult<Header<'a>> {
    let header = match raw_header.name {
      "Host" => Header::Host(slice_to_str(raw_header.value)?),
      "Authorization" => Header::Authorization(Authorization::parse(raw_header.value)?),
      "Referer" => Header::Referer(slice_to_str(raw_header.value)?),
      "Content-Type" => Header::ContentType(MimeType::parse(slice_to_str(raw_header.value)?)?),
      "Content-Length" => Header::ContentLength(parse_u64(slice_to_str(raw_header.value)?)?),
      "Range" => Header::Range(ContentRange::parse(slice_to_str(raw_header.value)?)?),
      "If-None-Match" => Header::IfNoneMatch(ETagMatch::parse(raw_header.value)?),
      _ => Header::Other(raw_header)
    };
    Ok(header)
  }
}

