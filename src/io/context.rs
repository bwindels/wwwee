//use std;
//use std::path::Path;
//use super::token::AsyncToken;
use buffer::Buffer;
use buffer::pool::BorrowError;

pub trait Context<'a> {
  fn borrow_buffer(&self, size_hint: usize) -> Result<Buffer<'a>, BorrowError>;
}

pub mod mio {
  use mio;
  use buffer::pool::{BufferPool, BorrowError};
  use buffer::Buffer;
  use io::token::ConnectionId;
  
  pub struct Context<'a, 'b> {
    poll: &'b mio::Poll,
    buffer_pool: &'a BufferPool<'a>,
    conn_id: ConnectionId
  }

  impl<'a: 'b, 'b> Context<'a, 'b> {
    pub fn new(poll: &'b mio::Poll, buffer_pool: &'a BufferPool<'a>, conn_id: ConnectionId) -> Context<'a, 'b> {
      Context {poll, buffer_pool, conn_id}
    }
  }

  impl<'a: 'b, 'b> super::Context<'a> for Context<'a, 'b> {
    fn borrow_buffer(&self, size_hint: usize) -> Result<Buffer<'a>, BorrowError> {
      self.buffer_pool.borrow_buffer(size_hint)
    }
  /*
    pub fn read_file(&mut self, path: Path, range: Option<usize>, token: AsyncToken) -> std::io::Result<file::Reader> {
      Err(std::io::Error::new(std::io::ErrorKind::Other, "oh no!"))
    }
  */
  }
}
