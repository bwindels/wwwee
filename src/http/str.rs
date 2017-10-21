use super::error::*;
use std::str;
use split::BufferExt;

pub fn is_whitespace(b: u8) -> bool {
  b == 0x20 || b == 0x09
}

pub fn trim_left<P>(buffer: &[u8], predicate: P) -> &[u8] where P: Fn(u8) -> bool {
  if let Some(idx) = buffer.iter().position(|&b| !predicate(b)) {
    return &buffer[idx..];
  }
  buffer
}

pub fn trim_right<P>(buffer: &[u8], predicate: P) -> &[u8] where P: Fn(u8) -> bool {
  if let Some(idx) = buffer.iter().rposition(|&b| !predicate(b)) {
    return &buffer[..idx + 1];
  }
  buffer
}

pub fn trim<P>(buffer: &[u8], predicate: P) -> &[u8] where P: Fn(u8) -> bool {
  trim_right(trim_left(buffer, &predicate), &predicate)
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
  #[test]
  fn test_is_whitespace() {
    assert!(super::is_whitespace(b" "[0]));
    assert!(super::is_whitespace(b"\t"[0]));
  }
  #[test]
  fn test_trim_left() {
    assert_eq!(super::trim_left(b" \t hello ", super::is_whitespace), b"hello ");
  }
  #[test]
  fn test_trim_right() {
    assert_eq!(super::trim_right(b" hello \t ", super::is_whitespace), b" hello");
  }
  #[test]
  fn test_trim() {
    assert_eq!(super::trim(b" \t hello \t ", super::is_whitespace), b"hello");
  }
  
}