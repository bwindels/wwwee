use mio;
pub use self::AsyncToken;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ConnectionId(pub u32);

#[cfg(target_pointer_width = "64")]
pub use self::bit64::*;
#[cfg(target_pointer_width = "64")]
mod bit64 {
  #[derive(Clone, Copy, Debug, Default, PartialEq)]
  pub struct AsyncToken(pub u32);

  // TODO: deal with overflow here
  pub fn next_token(token: AsyncToken) -> AsyncToken {
    AsyncToken(token.0 + 1)
  }

  pub fn split_token(token: usize) -> (super::ConnectionId, AsyncToken) {
    let conn_id = super::ConnectionId ( (token >> 32) as u32 );
    let async_token = AsyncToken( (token & 0xFFFFFFFF) as u32 );
    (conn_id, async_token)
  }

  pub fn create_token(conn_id: super::ConnectionId, async_token: AsyncToken) -> usize {
    ((async_token.0 & 0xFFFFFFFF) as usize) | ((conn_id.0 as usize) << 32)
  }
}

#[cfg(target_pointer_width = "32")]
pub use self::bit32::*;
#[cfg(target_pointer_width = "32")]
mod bit32 {
  #[derive(Clone, Copy, Debug, Default, PartialEq)]
  pub struct AsyncToken(pub u16);

  // TODO: deal with overflow here
  pub fn next_token(token: AsyncToken) -> AsyncToken {
    AsyncToken(token.0 + 1)
  }

  pub fn split_token(token: usize) -> (super::ConnectionId, AsyncToken) {
    let conn_id = super::ConnectionId( (token >> 10) as u32 );
    let async_token = AsyncToken( (token & 0b11_1111_1111) as u16 );
    (conn_id, async_token)
  }

  pub fn create_token(conn_id: super::ConnectionId, async_token: AsyncToken) -> usize {
    ((async_token.0 & 0b11_1111_1111) as usize) | ((conn_id.0 as usize) << 10)
  }
}

impl ConnectionId {
  pub fn from_index(idx: usize) -> ConnectionId {
    ConnectionId( (idx as u32) + 1)
  }

  pub fn as_index(self) -> usize {
    self.0 as usize - 1
  }
}

#[derive(Clone, Copy)]
pub struct Token {
  token: usize
}

impl Token {
  pub fn from_mio_token(token: mio::Token) -> Token {
    Token { token: token.0 }
  }
  pub fn from_parts(conn_id: ConnectionId, async_token: AsyncToken) -> Token {
    Token { token: create_token(conn_id, async_token) }
  }

  pub fn as_mio_token(self) -> mio::Token {
    mio::Token(self.token)
  }

  pub fn async_token(self) -> AsyncToken {
    split_token(self.token).1
  }

  pub fn connection_id(self) -> ConnectionId {
    split_token(self.token).0
  }
}

pub struct AsyncTokenSource {
  counter: AsyncToken
}

impl AsyncTokenSource {
  
  pub fn starting_from(start_from: AsyncToken) -> AsyncTokenSource {
    AsyncTokenSource { counter: start_from }
  }

  pub fn alloc_async_token(&mut self) -> AsyncToken {
    self.counter = next_token(self.counter);
    self.counter
  }
}

#[cfg(test)]
mod tests {
  use super::{create_token, split_token, ConnectionId, AsyncToken};

  #[test]
  fn test_split() {
    let token : usize = create_token(ConnectionId(5), AsyncToken(2));
    let (conn_id, async_token) = split_token(token);
    assert_eq!(conn_id, ConnectionId(5));
    assert_eq!(async_token, AsyncToken(2));
  }

  #[test]
  fn test_max_async_token() {
    let token : usize = create_token(ConnectionId(5), AsyncToken(1023));
    let (conn_id, async_token) = split_token(token);
    assert_eq!(conn_id, ConnectionId(5));
    assert_eq!(async_token, AsyncToken(1023));
  }
}
