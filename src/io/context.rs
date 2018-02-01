use std;
use mio;
use super::{
  Token,
  AsyncTokenSource,
  ConnectionId,
  Registered,
  Register
};

pub struct Context<'a> {
  poll: &'a mio::Poll,
  conn_id: ConnectionId,
  token_source: &'a AsyncTokenSource
}

impl<'a> Context<'a> {
  pub fn new(poll: &'a mio::Poll, conn_id: ConnectionId, token_source: &'a AsyncTokenSource) -> Context<'a> {
    Context {poll, conn_id, token_source}
  }

  pub fn register<R: Register>(&self, registerable: R) -> std::io::Result<Registered<R>> {
    let token = self.alloc_token();
    let registered_handler = Registered::register(registerable, token, &self.poll)?;
    Ok(registered_handler)
  }

  pub fn deregister<R: Register>(&self, registerable: &mut R) -> std::io::Result<()> {
    registerable.deregister(&self.poll)
  }
  
  fn alloc_token(&self) -> Token {
    let async_token = self.token_source.alloc_async_token();
    Token::from_parts(self.conn_id, async_token)
  }
}
