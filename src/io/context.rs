//use std;
use mio;
//use std::path::Path;
//use super::token::AsyncToken;
use buffer::pool::{BufferPool, BorrowError};
use buffer::Buffer;
use io::token::ConnectionId;

pub struct Context<'a, 'b: 'a> {
  poll: &'a mio::Poll,
  buffer_pool: &'b BufferPool<'b>,
  conn_id: ConnectionId
}

impl<'a, 'b: 'a> Context<'a, 'b> {
  pub fn new(poll: &'a mio::Poll, buffer_pool: &'b BufferPool<'b>, conn_id: ConnectionId) -> Context<'a, 'b> {
    Context {poll, buffer_pool, conn_id}
  }
/*
  pub fn read_file(&mut self, path: Path, range: Option<usize>, token: AsyncToken) -> std::io::Result<file::Reader> {
    Err(std::io::Error::new(std::io::ErrorKind::Other, "oh no!"))
  }
*/
  pub fn borrow_buffer(&self, size_hint: usize) -> Result<Buffer<'b>, BorrowError> {
    self.buffer_pool.borrow_buffer(size_hint)
  }
}
