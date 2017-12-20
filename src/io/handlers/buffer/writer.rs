use buffer::Buffer;
use io::{Handler, AsyncToken, Context};
use std::io::{Write, ErrorKind};

pub struct BufferWriter<W> {
  buffer: Buffer,
  bytes_written: usize,
  writer: W
}

impl<'a, W: Write> BufferWriter<W> {
  pub fn new(buffer: Buffer, writer: W) -> BufferWriter<W> {
    BufferWriter { buffer, writer, bytes_written: 0 }
  }
}

impl<W: Write> Handler<()> for BufferWriter<W> {

  fn writable(&mut self, _: AsyncToken, _: &Context) -> Option<()> {
    let slice_to_write = &self.buffer.as_slice()[self.bytes_written ..];
    match self.writer.write(slice_to_write) {
      Ok(bytes_written) => {
        self.bytes_written += bytes_written;
        if self.bytes_written == self.buffer.len() {
          Some( () )
        }
        else {
          None
        }
      },
      Err(err) => {
        match err.kind() {
          ErrorKind::Interrupted |
          ErrorKind::WouldBlock => None,
          _ => Some( () )
        }
      }
    }
  }
  
}

