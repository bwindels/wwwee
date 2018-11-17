use http::{RequestResult, RequestError};
use http::str::{try_split_two_mut, trim_mut, is_whitespace, slice_to_str};
use encoding::base64;

pub struct BasicCredentials<'a> {
	pub user: &'a str,
	pub password: &'a str
}

pub enum Authorization<'a> {
  Basic(BasicCredentials<'a>),
  Digest(&'a str),
  Bearer(&'a str),  //should be mut to decode inline whatever the format is?
  Other(&'a str, &'a mut [u8])
}

impl<'a> Authorization<'a> {
	pub fn parse(header_value: &'a mut [u8]) -> RequestResult<Authorization<'a>> {
    let (auth_kind, credentials) = try_split_two_mut(header_value, b" ");
    let credentials = credentials.ok_or(RequestError::InvalidHeader)?;
    let auth_kind = slice_to_str(trim_mut(auth_kind, is_whitespace))?;
    let credentials = trim_mut(credentials, is_whitespace);
    match auth_kind {
      "Basic" => {
        let decoded = base64::decode(credentials).ok_or(RequestError::InvalidHeader)?;
        let (user, password) = try_split_two_mut(decoded, b":");
        let password = password.ok_or(RequestError::InvalidHeader)?;
        let user = slice_to_str(user)?;
        let password = slice_to_str(password)?;
        Ok(Authorization::Basic( BasicCredentials { user, password } ))
      },
      "Digest" => {
        Ok(Authorization::Digest(slice_to_str(credentials)?))
      },
      "Bearer" => {
        Ok(Authorization::Bearer(slice_to_str(credentials)?))
      },
      _ => {
        Ok(Authorization::Other(auth_kind, credentials))
      }
    }
	}
}

#[cfg(test)]
mod tests {

  use test_helpers::copy_str;
  use super::*;

  #[test]
  fn test_basic_auth() {
    let mut buffer = [0u8; 18];
    copy_str(&mut buffer, b"Basic Zm9vOmJhcg==");
    let auth = Authorization::parse(&mut buffer).unwrap();
    if let Authorization::Basic(ref credentials) = auth {
      assert_eq!(credentials.user, "foo");
      assert_eq!(credentials.password, "bar");
    } else {
      assert!(false);
    }
  }
}
