use std;
use mio;
use super::{
  Token,
  AsyncTokenSource,
  ConnectionId,
  Registered,
  AsyncSource,
  EventSource
};
// TODO move this away from io and maybe to server or connection module?
// it's not really generic io, but geared towards a connection (ConnectionId, Socket)
pub struct Context<'a> {
  poll: &'a mio::Poll,
  conn_id: ConnectionId,
  token_source: &'a AsyncTokenSource,
  socket: &'a mut Socket
}

impl<'a> Context<'a>
{
  pub fn new(poll: &'a mio::Poll, conn_id: ConnectionId, token_source: &'a AsyncTokenSource, socket: &'a mut Socket) -> Context<'a> {
    Context {poll, conn_id, token_source, socket}
  }

  pub fn with_wrapped_socket(&self, socket: &'a mut Socket) -> Context<'a>
  {
    Context {
      poll: self.poll,
      conn_id: self.conn_id,
      token_source: self.token_source,
      socket
    }
  }

  pub fn register<R: AsyncSource>(&self, registerable: R) -> std::io::Result<Registered<R>> {
    let token = self.alloc_token();
    let registered_handler = Registered::register(registerable, token, &self.poll)?;
    Ok(registered_handler)
  }

  pub fn deregister<R: AsyncSource>(&self, registerable: &mut R) -> std::io::Result<()> {
    registerable.deregister(&self.poll)
  }

  pub fn socket<'b>(&'b mut self) -> &'b mut (Socket + 'b) {
    &mut self.socket
  }
  
  fn alloc_token(&self) -> Token {
    let async_token = self.token_source.alloc_async_token();
    Token::from_parts(self.conn_id, async_token)
  }
}

pub trait Socket : EventSource + std::io::Read + ::io::ReadSizeHint + std::io::Write {}
impl<S: EventSource + std::io::Read + ::io::ReadSizeHint + std::io::Write> Socket for S {}
