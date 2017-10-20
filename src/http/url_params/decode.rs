use http::url_decode::{
  url_decode,
  url_decode_and_move_1,
  contains_percent_values
};
use split::{BufferExt, buffer_split_mut};
use http::error::{RequestError, RequestResult};

fn decode_and_mark_params(buffer: &mut [u8]) -> RequestResult<()> {
  for param in buffer_split_mut(buffer, b"&") {
    let (name, value) = try_split_two_mut(param, b"=");
    decode_and_mark_component(name)?;
    if let Some(value) = value {
      decode_and_mark_component(value)?;
    }
  }
  Ok( () )
}

fn decode_and_mark_component(mut component: &mut [u8]) -> RequestResult<()> {
  let end_idx = if contains_percent_values(component) {

    let decoded_len = {
      let decoded = url_decode_and_move_1(&mut component);
      if contains_nul_char(decoded) {
        return Err(RequestError::UrlEncodedNul)
      }
      decoded.len()
    };

    component[0] = 0;
    decoded_len + 1
  }
  else {
    let decoded = url_decode(&mut component);
    if contains_nul_char(decoded) {
      return Err(RequestError::UrlEncodedNul)
    }
    decoded.len()
  };

  for i in end_idx .. component.len() {
    component[i] = 0;
  }

  return Ok( () )
}

fn contains_nul_char(buffer: &[u8]) -> bool {
  buffer.iter().find(|&&b| b == 0u8).is_some()
}

fn try_split_two_mut<'a>(buffer: &'a mut [u8], operator: &[u8]) -> (&'a mut [u8], Option<&'a mut [u8]>) {
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
  use http::RequestError;
  #[test]
  fn test_name_value_nonpercent() {
    let mut buffer = [0u8; 11];
    copy_str(&mut buffer, b"he+lo=wo+ld");
    super::decode_and_mark_params(&mut buffer).unwrap();
    assert_eq!(&buffer, b"he lo=wo ld");
  }
  #[test]
  fn test_name_value_percent() {
    let mut buffer = [0u8; 20];
    copy_str(&mut buffer, b"hel%3dlo=wo%26%3drld");
    super::decode_and_mark_params(&mut buffer).unwrap();
    assert_eq!(&buffer, b"\0hel=lo\0=\0wo&=rld\0\0\0");
  }

  #[test]
  fn test_multiple_pairs() {
    let mut buffer = [0u8; 32];
    copy_str(&mut buffer, b"hel%3dlo=wo%26%3drld&he+lo=wo+ld");
    super::decode_and_mark_params(&mut buffer).unwrap();
    assert_eq!(&buffer, b"\0hel=lo\0=\0wo&=rld\0\0\0&he lo=wo ld");
  }

  #[test]
  fn test_contains_nul_error() {
    let mut buffer = [0u8; 1];
    copy_str(&mut buffer, b"\0");
    let result = super::decode_and_mark_params(&mut buffer);
    assert_eq!(result, Err(RequestError::UrlEncodedNul));
  }
}
