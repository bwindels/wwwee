//use std;
//use std::path::Path;
//use super::token::AsyncToken;

use mio;
use io::token::ConnectionId;

pub struct Context<'a> {
  poll: &'a mio::Poll,
  conn_id: ConnectionId
}

impl<'a> Context<'a> {
  pub fn new(poll: &'a mio::Poll, conn_id: ConnectionId) -> Context<'a> {
    Context {poll, conn_id}
  }

/*
  pub fn read_file(&mut self, path: Path, range: Option<usize>, token: AsyncToken) -> std::io::Result<file::Reader> {
    Err(std::io::Error::new(std::io::ErrorKind::Other, "oh no!"))
  }
*/
}
