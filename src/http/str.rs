use super::error::*;
use std::str;
use split::BufferExt;

pub fn is_whitespace(b: u8) -> bool {
  b == 0x20 || b == 0x09
}

pub fn trim_left_mut<P>(buffer: &mut [u8], predicate: P) -> &mut [u8] where P: Fn(u8) -> bool {
  if let Some(idx) = buffer.iter().position(|&b| !predicate(b)) {
    return &mut buffer[idx..];
  }
  buffer
}

pub fn trim_right_mut<P>(buffer: &mut [u8], predicate: P) -> &mut [u8] where P: Fn(u8) -> bool {
  if let Some(idx) = buffer.iter().rposition(|&b| !predicate(b)) {
    return &mut buffer[..idx + 1];
  }
  buffer
}

pub fn trim_mut<P>(buffer: &mut [u8], predicate: P) -> &mut [u8] where P: Fn(u8) -> bool {
  trim_right_mut(trim_left_mut(buffer, &predicate), &predicate)
}


pub fn slice_to_str(slice: &[u8]) -> RequestResult<&str> {
  str::from_utf8(slice).or(Err(RequestError::InvalidEncoding))
}

pub fn try_split_two_mut<'a>(buffer: &'a mut [u8], operator: &[u8]) -> (&'a mut [u8], Option<&'a mut [u8]>) {
  if let Some(operator_idx) = buffer.position(operator) {
    let (lhs, remainder) = buffer.split_at_mut(operator_idx);
    let rhs = &mut remainder[ 1 .. ];
    (lhs, Some(rhs) )
  }
  else {
    (buffer, None)
  }
}


#[cfg(test)]
mod tests {
  use test_helpers::copy_str;
  use super::*;

  #[test]
  fn test_is_whitespace() {
    assert!(is_whitespace(b" "[0]));
    assert!(is_whitespace(b"\t"[0]));
  }
  #[test]
  fn test_trim_left_mut() {
    let mut buffer = [0u8; 9];
    copy_str(&mut buffer, b" \t hello ");
    assert_eq!(&trim_left_mut(&mut buffer, is_whitespace), b"hello ");
  }
  #[test]
  fn test_trim_right_mut() {
    let mut buffer = [0u8; 9];
    copy_str(&mut buffer, b" hello \t ");
    assert_eq!(&trim_right_mut(&mut buffer, is_whitespace), b" hello");
  }
  #[test]
  fn test_trim_mut() {
    let mut buffer = [0u8; 11];
    copy_str(&mut buffer, b" \t hello \t ");
    assert_eq!(&trim_mut(&mut buffer, is_whitespace), b"hello");
  }
  
}
