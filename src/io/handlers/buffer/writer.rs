use buffer::Buffer;
use io::{Handler, Event, AsyncSource, Registered, Context};
use io::handlers::{send_buffer, SendResult};
use std::io::Write;
use std::ops::DerefMut;

pub struct BufferWriter<W> {
  buffer: Buffer,
  bytes_written: usize,
  writer: Registered<W>
}

impl<'a, W: Write> BufferWriter<W> {
  pub fn new(writer: Registered<W>, buffer: Buffer) -> BufferWriter<W> {
    BufferWriter { buffer, writer, bytes_written: 0 }
  }

  pub fn into_writer(self) -> Registered<W> {
    self.writer
  }
}

impl<W: Write + AsyncSource> Handler<usize> for BufferWriter<W> {

  fn handle_event(&mut self, event: &Event, _ctx: &Context) -> Option<usize> {
    if !self.writer.is_source_of(event) {
      return None;
    }

    let slice_to_write = &self.buffer.as_slice()[self.bytes_written ..];

    match send_buffer(self.writer.deref_mut(), slice_to_write) {
      SendResult::WouldBlock(bytes_written) => {
        self.bytes_written += bytes_written;
        None
      },
      SendResult::Consumed => {
        self.bytes_written += slice_to_write.len();
        Some( self.bytes_written )
      },
      SendResult::IoError(_) => {
        Some( self.bytes_written )
      }
    }
  }
  
}

