use buffer::Buffer;
use io::{Handler, Event, Context};
use io::handlers::{send_buffer, IoReport};

pub struct BufferResponder {
  buffer: Buffer,
  bytes_written: usize
}

impl<'a> BufferResponder {
  pub fn new(buffer: Buffer) -> BufferResponder {
    BufferResponder { buffer, bytes_written: 0 }
  }
}

impl Handler<usize> for BufferResponder {

  fn handle_event(&mut self, event: &Event, ctx: &mut Context) -> Option<usize> {
    let mut socket = ctx.socket();
    if !socket.is_source_of(event) {
      return None;
    }

    let slice_to_write = &self.buffer.as_slice()[self.bytes_written ..];

    match send_buffer(&mut socket, slice_to_write) {
      Ok(IoReport::Partial(bytes_written)) => {
        self.bytes_written += bytes_written;
        None
      },
      Ok(IoReport::Complete(bytes_written)) => {
        Some( self.bytes_written + bytes_written )
      },
      Ok(IoReport::Empty) => None,
      Err(_) => {
        Some( self.bytes_written )
      }
    }
  }
  
}

