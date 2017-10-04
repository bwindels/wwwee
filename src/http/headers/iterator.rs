use super::Header;
use http::RequestResult;


pub struct HeaderIterator<'a> {
  headers: &'a str
}

impl<'a> Iterator for HeaderIterator<'a> {
  type Item = RequestResult<Header<'a>>;

  fn next(&mut self) -> Option<Self::Item> {
    None
  }
}