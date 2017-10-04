#[derive(Debug)]
pub enum RequestError {
  InvalidRequestLine,
  InvalidHeader,
  InvalidEncoding
}

pub type RequestResult<T> = Result<T, RequestError>;