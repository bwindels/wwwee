#[derive(Debug)]
pub enum ParseError {
  InvalidRequestLine,
  InvalidHeader
}

pub type ParseResult<T> = Result<T, ParseError>;