use http::url_decode::{
  url_decode,
  url_decode_and_move_1,
  contains_percent_values
};
use http::str::try_split_two_mut;
use split::buffer_split_mut;
use http::error::{RequestError, RequestResult};
use std::str::from_utf8;

pub fn decode_and_mark_params(buffer: &mut [u8]) -> RequestResult<()> {
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
      validate_component(decoded)?;
      decoded.len()
    };

    component[0] = 0;
    decoded_len + 1
  }
  else {
    let decoded = url_decode(&mut component);
    validate_component(decoded)?;
    decoded.len()
  };

  for i in end_idx .. component.len() {
    component[i] = 0;
  }

  return Ok( () )
}

fn validate_component(buffer: &[u8]) -> RequestResult<()> {
  if buffer.iter().find(|&&b| b == 0u8).is_some() {
    Err(RequestError::UrlEncodedNul)
  }
  else if from_utf8(buffer).is_err() {
    Err(RequestError::InvalidEncoding)
  }
  else {
    Ok( () )
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
