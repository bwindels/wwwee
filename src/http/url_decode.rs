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

pub fn url_decode(buffer: &mut [u8]) -> &mut [u8] {
  let mut write_idx = 0usize;
  let mut read_idx = 0usize;

  while read_idx < buffer.len() {
    match buffer[read_idx] {
      0x2B => buffer[write_idx] = 0x20, //+ to space
      0x25 => { //% encoded value
        let decoded_byte = if let (Some(upperhex), Some(lowerhex))
          = (buffer.get(read_idx + 1), buffer.get(read_idx + 2))
        {
          hex_to_byte(*upperhex, *lowerhex)
        }
        else {
          None
        };

        if let Some(decoded_byte) = decoded_byte {
          buffer[write_idx] = decoded_byte;
          read_idx += 2;
        }
        else if write_idx != read_idx {
          buffer[write_idx] = 0x25; //%
        }
      },
      _ => {
        if write_idx != read_idx {
          buffer[write_idx] = buffer[read_idx];
        }
      }
    };
    write_idx += 1;
    read_idx += 1;
  }
  &mut buffer[.. write_idx]
}

#[cfg(test)]
mod tests {

  use std::str;

  fn copy_str(dst: &mut [u8], src: &[u8]) {
    assert_eq!(src.len(), dst.len());
    let mut src_it = src.iter();
    for mut d in dst {
      *d = *src_it.next().unwrap();
    }
  }

  #[test]
  fn test_plus_space() {
    let mut buffer = [0u8; 11];
    copy_str(&mut buffer, b"hello+world");
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
    copy_str(&mut buffer, b"hello%20world");
    let decoded = super::url_decode(&mut buffer);
    assert_eq!(decoded, b"hello world");
  }


  #[test]
  fn test_ff_byte() {
    let mut buffer = [0u8; 3];
    {
      copy_str(&mut buffer, b"%FF");
      let decoded = super::url_decode(&mut buffer);
      assert_eq!(decoded, [0xFFu8]);
    }
    {
      copy_str(&mut buffer, b"%ff");
      let decoded = super::url_decode(&mut buffer);
      assert_eq!(decoded, [0xFFu8]);
    }
  }

  #[test]
  fn test_aa_byte() {
    let mut buffer = [0u8; 3];
    {
      copy_str(&mut buffer, b"%AA");
      let decoded = super::url_decode(&mut buffer);
      assert_eq!(decoded, [0xAAu8]);
    }
    {
      copy_str(&mut buffer, b"%aa");
      let decoded = super::url_decode(&mut buffer);
      assert_eq!(decoded, [0xAAu8]);
    }
  }

  #[test]
  fn test_00_byte() {
    let mut buffer = [0u8; 3];
    copy_str(&mut buffer, b"%00");
    let decoded = super::url_decode(&mut buffer);
    assert_eq!(decoded, [0x00u8]);
  }

  #[test]
  fn test_multiple_percent_encoded() {
    let mut buffer = [0u8; 22];
    copy_str(&mut buffer, b"%31%32%33+to+%61%62%63");
    let decoded = super::url_decode(&mut buffer);
    assert_eq!(decoded, b"123 to abc");
  }

  #[test]
  fn test_no_encoded_content() {
    let mut buffer = [0u8; 5];
    copy_str(&mut buffer, b"hello");
    let decoded = super::url_decode(&mut buffer);
    assert_eq!(decoded, b"hello");
  }

  #[test]
  fn test_percent_at_end_preserved() {
    let mut buffer = [0u8; 6];
    copy_str(&mut buffer, b"hello%");
    let decoded = super::url_decode(&mut buffer);
    assert_eq!(decoded, b"hello%");

    let mut buffer = [0u8; 7];
    copy_str(&mut buffer, b"hello%5");
    let decoded = super::url_decode(&mut buffer);
    assert_eq!(decoded, b"hello%5");
  }

  #[test]
  fn test_invalid_percent_encoding_preserved() {
    let mut buffer = [0u8; 13];
    copy_str(&mut buffer, b"hello%GFworld");
    let decoded = super::url_decode(&mut buffer);
    assert_eq!(decoded, b"hello%GFworld");
  }
  
}