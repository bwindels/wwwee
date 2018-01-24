use buffer::Buffer;
use io::{Handler, AsyncToken, Context};
use io::handlers::{send_buffer, SendResult};
use std::io::Write;

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

    match send_buffer(&mut self.writer, slice_to_write) {
      SendResult::WouldBlock(bytes_written) => {
        self.bytes_written += bytes_written;
        None
      },
      SendResult::Consumed => {
        self.bytes_written += slice_to_write.len();
        Some( () )
      },
      SendResult::IoError(_) => {
        Some( () )
      }
    }
  }
  
}

