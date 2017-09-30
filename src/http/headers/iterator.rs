use super::Header;
use error::ParseResult;


pub struct HeaderIterator<'a> {
  headers: &'a str
}

impl<'a> Iterator for HeaderIterator<'a> {
  type Item = ParseResult<Header<'a>>;

  fn next(&mut self) -> Option<Self::Item> {
    None
  }
}