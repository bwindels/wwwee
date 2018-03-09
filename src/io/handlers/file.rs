use io::sources::file::Reader;
use io::{Event, Handler, Registered, Context, EventSource};
use io::handlers::{send_buffer, IoReport};
use std;
use std::io::Write;

pub struct FileResponder {
  reader: Registered<Reader>,
  total_bytes_sent: usize,
  buffer_bytes_sent: usize,
  socket_writeable: bool
}

impl FileResponder {

  pub fn start(mut reader: Registered<Reader>) -> std::io::Result<FileResponder> {
    reader.try_queue_read()?; //queue initial read to start getting events
    Ok(FileResponder {
      reader,
      total_bytes_sent: 0,
      buffer_bytes_sent: 0,
      socket_writeable: true
    })
  }

  fn send_and_request_data(&mut self, socket: &mut Write) -> Option<usize> {
    // TODO: clean this op with higher level functions
    let need_more_data = if let Ok(buffer) = self.reader.try_get_read_bytes() {
      let mut remaining_bytes = &buffer[self.buffer_bytes_sent ..];
      match send_buffer(socket, remaining_bytes) {
        Ok(IoReport::Partial(bytes_written)) => {
          self.socket_writeable = false;
          self.buffer_bytes_sent += bytes_written;
          false
        },
        Ok(IoReport::Complete(bytes_written)) => {
          self.buffer_bytes_sent += bytes_written;
          true
        },
        Ok(IoReport::Empty) => false,
        Err(_) => {
          return Some(self.total_bytes_sent + self.buffer_bytes_sent);
        }
      }
    }
    else {
      false
    };

    if need_more_data {
      self.total_bytes_sent += self.buffer_bytes_sent;
      self.buffer_bytes_sent = 0;

      match self.reader.try_queue_read() {
        Ok(false) | //eof
        Err(_) => Some(self.total_bytes_sent),
        Ok(true) => None
      }
    }
    else {
      None
    }
  }

  pub fn into_reader(self) -> Registered<Reader> {
    self.reader
  }
}

impl Handler<usize> for FileResponder {
  fn handle_event(&mut self, event: &Event, ctx: &mut Context) -> Option<usize> {
    let mut socket = ctx.socket();
    //if the socket becomes readable, we don't care (http1)
    //or someone else should handle it (http2)
    //so in here we only handle reading from the file
    if socket.is_source_of(event) && event.kind().is_writable() {
      self.socket_writeable = true;
    }
    if socket.is_source_of(event) || self.reader.is_source_of(event) {
      self.send_and_request_data(&mut socket)
    }
    else {
      None
    }
  }
}
