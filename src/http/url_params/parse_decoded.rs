use std::ops::Range;

const NUL : u8 = 0x00;
const ASSIGN : u8 = 0x3D;
const AMPERSAND : u8 = 0x26;

#[derive(Clone, Copy)]
pub enum ComponentKind {
  Name,
  Value
}

use self::ComponentKind::*;

pub fn parse_decoded_component(buffer: &[u8], kind: ComponentKind) -> (Range<usize>, usize) {
  let mut start_idx = 0;
  let mut end_idx : Option<usize> = None;
  let mut starts_with_nul = false;
  let mut prev_nul = false;

  for i in 0 .. buffer.len() {
    let chr = buffer[i];
    match (chr, kind) {
      (NUL, _) => {
        if i == 0 {
          starts_with_nul = true;
          start_idx = 1;
        }
        else if starts_with_nul {
          end_idx = Some(i);
          break;
        }
      },
      (AMPERSAND, _) |
      (ASSIGN, Name) => {
        if (starts_with_nul && prev_nul) || !starts_with_nul {
          end_idx = Some(i);
          break;
        }
      },
      _ => ()
    }
    prev_nul = i != 0 && chr == NUL;
  }

  let end_idx = end_idx.unwrap_or(buffer.len());
  //find start of new component
  let next_start_idx = end_idx + find_next_start(&buffer[end_idx ..], kind);
  
  (start_idx .. end_idx, next_start_idx)
}

fn find_next_start(buffer: &[u8], kind: ComponentKind) -> usize {
  //swallow remaining \0 chars
  let next_start_idx = buffer.iter()
    .position(|&b| b != NUL)
    .unwrap_or(buffer.len());
  //swallow '=' in case of 'Name', and '&' in case of 'Value', when available
  let next_start_idx = next_start_idx + buffer.get(next_start_idx).map(|&chr| {
    match (chr, kind) {
      (ASSIGN, Name) |
      (AMPERSAND, Value) => 1,
      _ => 0
    }
  }).unwrap_or(0);

  next_start_idx
}

#[cfg(test)]
mod tests {
  use super::ComponentKind::*;

  #[test]
  fn test_param_name_with_no_value() {
    let buffer = b"foo&hello=bar";
    let (range, next_idx) = super::parse_decoded_component(buffer, Name);
    assert_eq!(&buffer[range], b"foo");
    assert_eq!(&buffer[next_idx .. ], b"&hello=bar"); 
  }
  #[test]
  fn test_param_nul_name() {
    let buffer = b"\0foo\0\0\0";
    let (range, next_idx) = super::parse_decoded_component(buffer, Name);
    assert_eq!(&buffer[range], b"foo");
    assert_eq!(&buffer[next_idx .. ], b""); 
  }
  #[test]
  fn test_param_nul_name_with_amp() {
    let buffer = b"\0foo\0\0\0&";
    let (range, next_idx) = super::parse_decoded_component(buffer, Name);
    assert_eq!(&buffer[range], b"foo");
    assert_eq!(&buffer[next_idx .. ], b"&"); 
  }
  #[test]
  fn test_param_nul_name_with_ampersand_and_assign() {
    let buffer = b"\0f=o&o\0\0\0&hello=bar";
    let (range, next_idx) = super::parse_decoded_component(buffer, Name);
    assert_eq!(&buffer[range], b"f=o&o");
    assert_eq!(&buffer[next_idx .. ], b"&hello=bar"); 
  }
  #[test]
  fn test_param_nul_value_with_ampersand_and_assign() {
    let buffer = b"\0f=o&o\0\0\0&hello=bar";
    let (range, next_idx) = super::parse_decoded_component(buffer, Value);
    assert_eq!(&buffer[range], b"f=o&o");
    assert_eq!(&buffer[next_idx .. ], b"hello=bar"); 
  }
  #[test]
  fn test_param_value_contains_equal() {
    let buffer = b"foo=";
    let (range, next_idx) = super::parse_decoded_component(buffer, Value);
    assert_eq!(&buffer[range], b"foo=");
    assert_eq!(&buffer[next_idx .. ], b"");

    let buffer = b"foo=&";
    let (range, next_idx) = super::parse_decoded_component(buffer, Value);
    assert_eq!(&buffer[range], b"foo=");
    assert_eq!(&buffer[next_idx .. ], b"");

    let buffer = b"foo=&bar";
    let (range, next_idx) = super::parse_decoded_component(buffer, Value);
    assert_eq!(&buffer[range], b"foo=");
    assert_eq!(&buffer[next_idx .. ], b"bar"); 
  }
  #[test]
  fn test_param_name_equals_no_value() {
    let buffer = b"foo=";
    let (range, next_idx) = super::parse_decoded_component(buffer, Name);
    assert_eq!(&buffer[range], b"foo");
    assert_eq!(&buffer[next_idx .. ], b""); 
  }
  #[test]
  fn test_param_empty_name_and_value() {
    let buffer = b"&bar";
    let (range, next_idx) = super::parse_decoded_component(buffer, Name);
    assert_eq!(&buffer[range], b"");
    assert_eq!(&buffer[next_idx .. ], buffer);
    let (range, next_idx) = super::parse_decoded_component(buffer, Value);
    assert_eq!(&buffer[range], b"");
    assert_eq!(&buffer[next_idx .. ], b"bar");
  }
  #[test]
  fn test_param_with_ampersand_or_assign_first() {
    let buffer = b"\0=\0";
    let (range, next_idx) = super::parse_decoded_component(buffer, Name);
    assert_eq!(&buffer[range], b"=");
    assert_eq!(&buffer[next_idx .. ], b""); 
  }
}