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



/*
  pub fn read_file(&mut self, path: Path, range: Option<usize>, token: AsyncToken) -> std::io::Result<file::Reader> {
    Err(std::io::Error::new(std::io::ErrorKind::Other, "oh no!"))
  }
*/
}
