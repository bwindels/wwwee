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

pub struct ContextFactory<'a> {
  poll: &'a mio::Poll,
  conn_id: ConnectionId,
  token_source: &'a mut AsyncTokenSource,
  uses_tls: bool
}

impl<'a> ContextFactory<'a> {
  pub fn set_tls(&mut self, uses_tls: bool) {
    self.uses_tls = uses_tls;
  }

  pub fn into_context<'s>(self, socket: &'s mut Socket)
    -> Context<'s>
    where 'a: 's
  {
    Context {
      poll: self.poll,
      conn_id: self.conn_id,
      token_source: self.token_source,
      uses_tls: self.uses_tls,
      socket
    }
  }
}

// TODO move this away from io and maybe to server or connection module?
// it's not really generic io, but geared towards a connection (ConnectionId, Socket)
pub struct Context<'a> {
  poll: &'a mio::Poll,
  conn_id: ConnectionId,
  token_source: &'a mut AsyncTokenSource,
  socket: &'a mut Socket,
  uses_tls: bool
}

impl<'a> Context<'a>
{
  pub fn new(poll: &'a mio::Poll, conn_id: ConnectionId, token_source: &'a mut AsyncTokenSource, socket: &'a mut Socket) -> Context<'a> {
    Context {poll, conn_id, token_source, socket, uses_tls: false}
  }

  pub fn register<R: AsyncSource>(&mut self, registerable: R) -> std::io::Result<Registered<R>> {
    let token = self.alloc_token();
    let registered_handler = Registered::register(registerable, token, &self.poll)?;
    Ok(registered_handler)
  }

  pub fn deregister<R: AsyncSource>(&self, registerable: &mut R) -> std::io::Result<()> {
    registerable.deregister(&self.poll)
  }

  pub fn socket(&mut self) -> &mut Socket {
    self.socket
  }

  pub fn as_socket_and_factory(&mut self) -> (&mut Socket, ContextFactory) {
    let factory = ContextFactory {
      poll: &self.poll,
      conn_id: self.conn_id,
      token_source: &mut self.token_source,
      uses_tls: self.uses_tls
    };
    (self.socket, factory)
  }

  pub fn uses_tls(&self) -> bool {
    self.uses_tls
  }

  fn alloc_token(&mut self) -> Token {
    let async_token = self.token_source.alloc_async_token();
    Token::from_parts(self.conn_id, async_token)
  }
}

pub trait Socket : EventSource + std::io::Read + ::io::ReadSizeHint + std::io::Write {}
impl<S: EventSource + std::io::Read + ::io::ReadSizeHint + std::io::Write> Socket for S {}
