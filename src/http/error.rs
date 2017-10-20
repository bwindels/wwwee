#[derive(Debug)]
pub enum RequestError {
  InvalidRequestLine,
  InvalidHeader,
  InvalidEncoding,
  UrlEncodedNul
}

pub type RequestResult<T> = Result<T, RequestError>;