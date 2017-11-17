use super::parse_decoded::{
  ComponentKind,
  parse_decoded_component
};
use super::decode::decode_and_mark_params;
use http::RequestResult;
use std::str;

pub struct UrlEncodedParams<'a> {
  decoded_params: &'a [u8]
}

impl<'a> UrlEncodedParams<'a> {
  pub fn decode_and_create(buffer: &'a mut [u8]) -> RequestResult<UrlEncodedParams<'a>> {
    //can't use map here because borrow checker doesn't
    //see buffer borrow end after call to decode_and_mark_params
    if let Err(e) = decode_and_mark_params(buffer) {
      Err(e)
    }
    else {
      Ok(UrlEncodedParams {decoded_params: buffer})
    }
  }

  pub fn iter(&self) -> UrlEncodedParamsIterator<'a> {
    UrlEncodedParamsIterator { remaining_params: self.decoded_params }
  }
}

pub struct UrlEncodedParamsIterator<'a> {
  remaining_params: &'a [u8]
}

impl<'a> Iterator for UrlEncodedParamsIterator<'a> {
  type Item = Param<'a>;

  fn next(&mut self) -> Option<Self::Item> {
    if self.remaining_params.len() == 0 {
      return None;
    }

    let (param, next_idx) = {
      let before_name = &self.remaining_params;
      let (name_range, name_next_idx) =
        parse_decoded_component(&before_name, ComponentKind::Name);
      let before_value = &before_name[name_next_idx ..];
      let (value_range, value_next_idx) =
        parse_decoded_component(&before_value, ComponentKind::Value);

      //name and value have been checked for utf8 validity beforehand
      //in `decode_and_mark_component` so we don't
      //have to return errors in the iterator
      let name = unsafe{ str::from_utf8_unchecked(&before_name[name_range]) };
      let value = unsafe{ str::from_utf8_unchecked(&before_value[value_range]) };
      (Param {name, value}, name_next_idx + value_next_idx)
    };

    self.remaining_params = &self.remaining_params[next_idx ..];

    Some(param)
  }
}

#[derive(Debug, PartialEq)]
pub struct Param<'a> {
  pub name: &'a str,
  pub value: &'a str
}

#[cfg(test)]
mod tests {
  use test_helpers::copy_str;
  use super::UrlEncodedParams;
  use http::RequestError;

  #[test]
  fn test_one_param() {
    let mut buffer = [0u8; 7];
    copy_str(&mut buffer, b"foo=bar");
    let params = UrlEncodedParams::decode_and_create(&mut buffer).unwrap();
    let mut iter = params.iter();

    let param = iter.next().unwrap();
    assert_eq!(param.name, "foo");
    assert_eq!(param.value, "bar");
    assert!(iter.next().is_none());
  }

  #[test]
  fn test_two_params() {
    let mut buffer = [0u8; 19];
    copy_str(&mut buffer, b"foo=bar&hello=world");
    let params = UrlEncodedParams::decode_and_create(&mut buffer).unwrap();
    let mut iter = params.iter();

    let first_param = iter.next().unwrap();
    assert_eq!(first_param.name, "foo");
    assert_eq!(first_param.value, "bar");

    let second_param = iter.next().unwrap();
    assert_eq!(second_param.name, "hello");
    assert_eq!(second_param.value, "world");

    assert!(iter.next().is_none());
  }

  #[test]
  fn test_one_percent_encoded_param() {
    let mut buffer = [0u8; 24];
    copy_str(&mut buffer, b"foo%5b%5d=bread%26butter");
    let params = UrlEncodedParams::decode_and_create(&mut buffer).unwrap();
    let mut iter = params.iter();

    let param = iter.next().unwrap();
    assert_eq!(param.name, "foo[]");
    assert_eq!(param.value, "bread&butter");
    assert_eq!(iter.next(), None);
  }

  #[test]
  fn test_two_percent_encoded_params() {
    let mut buffer = [0u8; 44];
    copy_str(&mut buffer, b"foo%5b%5d=bread%26butter&%3d%3d%3d=%26%26%26");
    let params = UrlEncodedParams::decode_and_create(&mut buffer).unwrap();
    let mut iter = params.iter();

    let param = iter.next().unwrap();
    assert_eq!(param.name, "foo[]");
    assert_eq!(param.value, "bread&butter");

    let param = iter.next().unwrap();
    assert_eq!(param.name, "===");
    assert_eq!(param.value, "&&&");
    
    assert_eq!(iter.next(), None);
  }

  #[test]
  fn test_iterate_twice() {
    let mut buffer = [0u8; 23];
    copy_str(&mut buffer, b"foo%5b%5d=bar&foo_len=1");
    let params = UrlEncodedParams::decode_and_create(&mut buffer).unwrap();

    for _ in 0 .. 2 {
      let mut iter = params.iter();
      
      let param = iter.next().unwrap();
      assert_eq!(param.name, "foo[]");
      assert_eq!(param.value, "bar");

      let param = iter.next().unwrap();
      assert_eq!(param.name, "foo_len");
      assert_eq!(param.value, "1");
    }
  }

  #[test]
  fn test_fail_on_nul() {
    let mut buffer = [0u8; 13];
    copy_str(&mut buffer, b"hello%00world");
    let result = UrlEncodedParams::decode_and_create(&mut buffer);
    assert_eq!(result.err(), Some(RequestError::UrlEncodedNul));
    let mut buffer = [0u8; 11];
    copy_str(&mut buffer, b"hello\0world");
    let result = UrlEncodedParams::decode_and_create(&mut buffer);
    assert_eq!(result.err(), Some(RequestError::UrlEncodedNul));
  }

  #[test]
  fn test_on_invalid_utf8() {
    let mut buffer = [0u8; 16];
    copy_str(&mut buffer, b"hello%C2%20world");
    let result = UrlEncodedParams::decode_and_create(&mut buffer);
    assert_eq!(result.err(), Some(RequestError::InvalidEncoding));
    let mut buffer = [0xC2, 0x20];
    let result = UrlEncodedParams::decode_and_create(&mut buffer);
    assert_eq!(result.err(), Some(RequestError::InvalidEncoding));
  }
  
}
