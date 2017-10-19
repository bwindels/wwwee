const PLUS : u8 = 0x2B;
const SPACE : u8 = 0x20;
const PERCENT : u8 = 0x25;

fn hex_decode_digit(c: u8) -> Option<u8> {
  match c {
    0x61 ... 0x66 => Some(10 + (c - 0x61)),  //a-f
    0x41 ... 0x46 => Some(10 + (c - 0x41)),  //A-F
    0x30 ... 0x39 => Some(c - 0x30),         //0-9
    _ => None
  }
}

fn hex_to_byte(upperdigit: u8, lowerdigit: u8) -> Option<u8> {
  if let (Some(upperbits), Some(lowerbits))
    = (hex_decode_digit(upperdigit), hex_decode_digit(lowerdigit))
  {
    Some((upperbits << 4) | lowerbits)
  }
  else {
    None
  }
}

/*
Starts writing the decoded value at index 1 instead of 0.
This would only be used if the buffer contains a
percent encoded value (check with `contains_percent_values`).
This leaves a byte at the beginning of the buffer to put a marker of some sort.
See the implementation of `UrlEncodedParams`.

If used on values without percent encoded values, it will trim off the last character.
The first byte is never written.
*/
pub fn url_decode_and_move_1(buffer: &mut [u8]) -> &mut [u8] {
  url_decode_with_offset_flag(buffer, true)
}

pub fn url_decode(buffer: &mut [u8]) -> &mut [u8] {
  url_decode_with_offset_flag(buffer, false)
}


fn url_decode_with_offset_flag(buffer: &mut [u8], write_offset_1: bool) -> &mut [u8] {
  if buffer.len() == 0 {
    return buffer;
  }

  let mut write_idx = if write_offset_1 {1usize} else {0usize};
  let offset = write_idx;
  let mut read_idx = 0usize;
  let mut byte = buffer[read_idx];

  while read_idx < buffer.len() {
    let new_byte = match byte {
      PLUS => SPACE,
      PERCENT => {
        let decoded_byte = if let (Some(upperhex), Some(lowerhex))
          = (buffer.get(read_idx + 1), buffer.get(read_idx + 2))
        {
          hex_to_byte(*upperhex, *lowerhex)
        }
        else {
          None
        };

        if let Some(decoded_byte) = decoded_byte {
          read_idx += 2;
          decoded_byte
        }
        else {
          PERCENT
        }
      },
      _ => byte
    };
    //advance and read new value
    read_idx += 1;
    if read_idx < buffer.len() {
      byte = buffer[read_idx];
    }
    //write new value to old index
    //since the new value is already read,
    //we can write to read_idx + 1 without
    //overwriting an unprocessed byte
    if write_idx < buffer.len() {
      buffer[write_idx] = new_byte;
      write_idx += 1;
    }
  }
  &mut buffer[ offset .. write_idx]
}

pub fn contains_percent_values(buffer: &[u8]) -> bool {
  let mut match_hex_digits = 0;
  for &byte in buffer {
    //currently matching hex digits after %?
    if match_hex_digits != 0 {
      //if the current byte isn't a hex digit, stop looking
      if hex_decode_digit(byte).is_none() {
        match_hex_digits = 0;
      }
      //else mark one as done, and see if we've finished to return true
      else {
        match_hex_digits -= 1;
        if match_hex_digits == 0 {
          return true;
        }
      }
    }
    //match the next 2 bytes as hex digits,
    //if they are, this slice contains percent encoded values!
    if byte == PERCENT {
      match_hex_digits = 2;
    }
  }
  return false;
}

#[cfg(test)]
mod tests {

  use std::str;
  use test_helpers;
  
  #[test]
  fn test_plus_space() {
    let mut buffer = [0u8; 11];
    test_helpers::copy_str(&mut buffer, b"hello+world");
    let decoded = super::url_decode(&mut buffer);
    assert_eq!(decoded, b"hello world");
  }

  #[test]
  fn test_empty() {
    let mut buffer = [0u8; 0];
    let decoded = super::url_decode(&mut buffer);
    assert_eq!(decoded, b"");
  }

  #[test]
  fn test_percent_encoded() {
    let mut buffer = [0u8; 13];
    test_helpers::copy_str(&mut buffer, b"hello%20world");
    let decoded = super::url_decode(&mut buffer);
    assert_eq!(decoded, b"hello world");
  }


  #[test]
  fn test_ff_byte() {
    let mut buffer = [0u8; 3];
    {
      test_helpers::copy_str(&mut buffer, b"%FF");
      let decoded = super::url_decode(&mut buffer);
      assert_eq!(decoded, [0xFFu8]);
    }
    {
      test_helpers::copy_str(&mut buffer, b"%ff");
      let decoded = super::url_decode(&mut buffer);
      assert_eq!(decoded, [0xFFu8]);
    }
  }

  #[test]
  fn test_aa_byte() {
    let mut buffer = [0u8; 3];
    {
      test_helpers::copy_str(&mut buffer, b"%AA");
      let decoded = super::url_decode(&mut buffer);
      assert_eq!(decoded, [0xAAu8]);
    }
    {
      test_helpers::copy_str(&mut buffer, b"%aa");
      let decoded = super::url_decode(&mut buffer);
      assert_eq!(decoded, [0xAAu8]);
    }
  }

  #[test]
  fn test_00_byte() {
    let mut buffer = [0u8; 3];
    test_helpers::copy_str(&mut buffer, b"%00");
    let decoded = super::url_decode(&mut buffer);
    assert_eq!(decoded, [0x00u8]);
  }

  #[test]
  fn test_multiple_percent_encoded() {
    let mut buffer = [0u8; 22];
    test_helpers::copy_str(&mut buffer, b"%31%32%33+to+%61%62%63");
    let decoded = super::url_decode(&mut buffer);
    assert_eq!(decoded, b"123 to abc");
  }

  #[test]
  fn test_double_percent_encoded() {
    let mut buffer = [0u8; 4];
    test_helpers::copy_str(&mut buffer, b"%%31");
    let decoded = super::url_decode(&mut buffer);
    assert_eq!(decoded, b"%1");
  }

  #[test]
  fn test_no_encoded_content() {
    let mut buffer = [0u8; 5];
    test_helpers::copy_str(&mut buffer, b"hello");
    let decoded = super::url_decode(&mut buffer);
    assert_eq!(decoded, b"hello");
  }

  #[test]
  fn test_percent_at_end_preserved() {
    let mut buffer = [0u8; 6];
    test_helpers::copy_str(&mut buffer, b"hello%");
    let decoded = super::url_decode(&mut buffer);
    assert_eq!(decoded, b"hello%");

    let mut buffer = [0u8; 7];
    test_helpers::copy_str(&mut buffer, b"hello%5");
    let decoded = super::url_decode(&mut buffer);
    assert_eq!(decoded, b"hello%5");
  }

  #[test]
  fn test_invalid_percent_encoding_preserved() {
    let mut buffer = [0u8; 13];
    test_helpers::copy_str(&mut buffer, b"hello%GFworld");
    let decoded = super::url_decode(&mut buffer);
    assert_eq!(decoded, b"hello%GFworld");
  }

  #[test]
  fn test_contains_percent_values() {
    assert!(super::contains_percent_values(b"hello%20world"));
    assert!(super::contains_percent_values(b"hello%20"));
    assert!(super::contains_percent_values(b"%%20"));
    assert!(!super::contains_percent_values(b"hello%"));
    assert!(!super::contains_percent_values(b"hello%5"));
    assert!(!super::contains_percent_values(b"hello+world"));
    assert!(!super::contains_percent_values(b"hello%GFworld"));
  }

  #[test]
  fn test_move_1_no_encoding() {
    let mut buffer = [0u8; 5];
    test_helpers::copy_str(&mut buffer, b"hello");
    {
      let decoded = super::url_decode_and_move_1(&mut buffer);
      assert_eq!(decoded, b"hell");
    }
    assert_eq!(&buffer,  b"hhell");
  }

    #[test]
  fn test_move_1_plus_encoding() {
    let mut buffer = [0u8; 11];
    test_helpers::copy_str(&mut buffer, b"hello+world");
    {
      let decoded = super::url_decode_and_move_1(&mut buffer);
      assert_eq!(decoded, b"hello worl");
    }
    assert_eq!(&buffer,  b"hhello worl");
  }

  #[test]
  fn test_move_1_middle_percent() {
    let mut buffer = [0u8; 13];
    test_helpers::copy_str(&mut buffer, b"hello%20world");
    {
      let decoded = super::url_decode_and_move_1(&mut buffer);
      assert_eq!(decoded, b"hello world");
    }
    assert_eq!(&buffer,  b"hhello worldd");
  }

  #[test]
  fn test_move_1_start_percent() {
    let mut buffer = [0u8; 8];
    test_helpers::copy_str(&mut buffer, b"%20hello");
    {
      let decoded = super::url_decode_and_move_1(&mut buffer);
      assert_eq!(decoded, b" hello");
    }
    assert_eq!(&buffer,  b"% helloo");
  }

  #[test]
  fn test_move_1_end_percent() {
    let mut buffer = [0u8; 8];
    test_helpers::copy_str(&mut buffer, b"hello%20");
    {
      let decoded = super::url_decode_and_move_1(&mut buffer);
      assert_eq!(decoded, b"hello ");
    }
    assert_eq!(&buffer,  b"hhello 0");
  }

  #[test]
  fn test_move_1_multiple_percent() {
    let mut buffer = [0u8; 30];
    test_helpers::copy_str(&mut buffer, b"hello%20world%20%26%20good+day");
    {
      let decoded = super::url_decode_and_move_1(&mut buffer);
      assert_eq!(decoded, b"hello world & good day");
    }
    assert_eq!(&buffer,  b"hhello world & good dayood+day");
  }
  
}