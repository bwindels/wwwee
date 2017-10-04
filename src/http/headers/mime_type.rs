use http::{RequestResult};

pub struct MimeType<'a> {
  pub mime_type: &'a str
}

impl<'a> MimeType<'a> {
  pub fn parse(header_value: &'a str) -> RequestResult<MimeType<'a>> {
    Ok(MimeType {mime_type: header_value})
  }
}