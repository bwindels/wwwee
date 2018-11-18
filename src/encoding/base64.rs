pub fn decode<'a>(src: &'a mut [u8]) -> Option<&'a mut [u8]> {
  if src.len() == 0 || src.len() % 4 != 0 {
    return None;
  }
  let mut previous_6bit = 0u8;
  let mut first_padding_idx : Option<usize> = None;
  for i in 0..src.len() {
    let current = src[i];
    let decoded_6bit = match current {
      65...90 => Some(current - 65),// A-Z,
      97...122 => Some(current - 97 + 26),// a-z,
      48...57 => Some(current + 4),  //0-9 (after 2x26 comes 52 which is 48 + 4)
      43 => Some(62), // +,
      47 => Some(63), // /,
      61 => {
        first_padding_idx = first_padding_idx.or(Some(i));
        Some(0)
      }, // = (padding)
      _ => None //invalid character
    }?;
    let decoded_8bit = match i % 4 {
      1 => Some(previous_6bit << 2 | (decoded_6bit & 0b11_0000) >> 4),
      2 => Some(previous_6bit << 4 | (decoded_6bit & 0b11_1100) >> 2),
      3 => Some(previous_6bit << 6 | (decoded_6bit & 0b11_1111)),
      _ => None,  //0
    };
    if let Some(d) = decoded_8bit {
      let index = i - (i / 4) - 1;
      src[index] = d;
    }
    previous_6bit = decoded_6bit;
  }
  let input_len = first_padding_idx.unwrap_or(src.len());
  if (src.len() - input_len) == 3 {
    return None;
  }
  // 3/4 of input_len as very 4th char doestn't yield a byte
  // for every 4 bytes, 3 output bytes
  // 3 input bytes, 2 ouput bytes
  // 2 input bytes, 1 output byte
  // 1 input byte, invalid
  let output_len = ((input_len / 4) * 3) + (input_len % 4).saturating_sub(1);
  Some(&mut src[0 .. output_len])
}

#[cfg(test)]
mod tests {

  use super::decode;
  use test_helpers;

  #[test]
  fn test_decode_invalid_char() {
    let mut buffer = [0u8; 1];
    test_helpers::copy_str(&mut buffer, b"?");
    assert_eq!(decode(&mut buffer), None);
  }

  #[test]
  fn test_decode_invalid_len() {
    let mut buffer = [0u8; 4];
    test_helpers::copy_str(&mut buffer, b"YWJj");
    assert_eq!(decode(&mut buffer[ 0 .. 0]), None);
    assert_eq!(decode(&mut buffer[.. 1]), None);
    assert_eq!(decode(&mut buffer[.. 2]), None);
    assert_eq!(decode(&mut buffer[.. 3]), None);
  }

  #[test]
  fn test_decode_invalid_padding() {
    let mut buffer = [0u8; 4];
    test_helpers::copy_str(&mut buffer, b"a===");
    assert_eq!(decode(&mut buffer), None);
  }

  #[test]
  fn test_decode_short() {
    let mut buffer = [0u8; 4];
    test_helpers::copy_str(&mut buffer, b"YQ==");
    assert_eq!(decode(&mut buffer).unwrap() as &[u8], b"a");
    test_helpers::copy_str(&mut buffer, b"YWI=");
    assert_eq!(decode(&mut buffer).unwrap() as &[u8], b"ab");
    test_helpers::copy_str(&mut buffer, b"YWJj");
    assert_eq!(decode(&mut buffer).unwrap() as &[u8], b"abc");
  }

  #[test]
  fn test_decode_medium() {
    let mut buffer = [0u8; 8];
    test_helpers::copy_str(&mut buffer, b"YWJjZA==");
    assert_eq!(decode(&mut buffer).unwrap() as &[u8], b"abcd");
    test_helpers::copy_str(&mut buffer, b"YWJjZGU=");
    assert_eq!(decode(&mut buffer).unwrap() as &[u8], b"abcde");
    test_helpers::copy_str(&mut buffer, b"YWJjZGVm");
    assert_eq!(decode(&mut buffer).unwrap() as &[u8], b"abcdef");
  }

  #[test]
  fn test_decode_hello_world() {
    let mut buffer = [0u8; 16];
    test_helpers::copy_str(&mut buffer, b"aGVsbG8gd29ybGQ=");
    assert_eq!(decode(&mut buffer).unwrap() as &[u8], b"hello world");
    test_helpers::copy_str(&mut buffer, b"aGVlbGxvIHdvcmxk");
    assert_eq!(decode(&mut buffer).unwrap() as &[u8], b"heello world");
  }
}
