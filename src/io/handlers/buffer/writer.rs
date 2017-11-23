use buffer::Buffer;
use io::{Handler, OperationState, AsyncToken, Context};
use std::io::{Write, ErrorKind};

pub struct BufferWriter<'a, W> {
  buffer: Buffer<'a>,
  bytes_written: usize,
  writer: W
}

impl<'a, W: Write> BufferWriter<'a, W> {
  pub fn new(buffer: Buffer<'a>, writer: W) -> BufferWriter<'a, W> {
    BufferWriter { buffer, writer, bytes_written: 0 }
  }
}

impl<'a, W: Write> Handler<usize> for BufferWriter<'a, W> {

  fn writable(&mut self, _: AsyncToken, _: &Context) -> OperationState<usize> {
    let slice_to_write = &self.buffer.as_slice()[self.bytes_written ..];
    match self.writer.write(slice_to_write) {
      Ok(bytes_written) => {
        self.bytes_written += bytes_written;
        if self.bytes_written == self.buffer.len() {
          OperationState::Finished(self.bytes_written)
        }
        else {
          OperationState::InProgress
        }
      },
      Err(err) => {
        match err.kind() {
          ErrorKind::Interrupted |
          ErrorKind::WouldBlock => OperationState::InProgress,
          _ => OperationState::Finished(self.bytes_written)
        }
      }
    }
  }
  
}

