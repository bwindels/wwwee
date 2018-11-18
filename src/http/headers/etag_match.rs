use http::str::{trim_mut, is_whitespace, slice_to_str};
use http::RequestResult;

pub enum ETagMatch<'a> {
  ETag(&'a str),
  Any
}

impl<'a> ETagMatch<'a> {
  pub fn parse(header_value: &'a mut [u8]) -> RequestResult<ETagMatch<'a>> {
    let value = trim_mut(header_value, is_whitespace);
    let value = trim_mut(value, |b| b == 34);  // trim "
    if value == b"*" {
      Ok(ETagMatch::Any)
    } else {
      Ok(ETagMatch::ETag(slice_to_str(value)?))
    }
  }
}
