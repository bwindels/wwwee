use io::sources::file::Reader;
use io::{Event, AsyncSource, Handler, Registered, Context};
use io::handlers::{send_buffer, SendResult};
use std::io::Write;
use std;
use std::ops::DerefMut;

pub struct FileResponder<W> {
  reader: Registered<Reader>,
  socket: Registered<W>,
  total_bytes_sent: usize,
  buffer_bytes_sent: usize,
  socket_writeable: bool
}

impl<W: Write> FileResponder<W> {

  pub fn start(socket: Registered<W>, mut reader: Registered<Reader>) -> std::io::Result<FileResponder<W>> {
    reader.try_queue_read()?; //queue initial read to start getting events
    Ok(FileResponder {
      reader,
      socket,
      total_bytes_sent: 0,
      buffer_bytes_sent: 0,
      socket_writeable: true
    })
  }

  fn send_and_request_data(&mut self) -> Option<usize> {
    let need_more_data = if let Ok(buffer) = self.reader.try_get_read_bytes() {
      let mut remaining_bytes = &buffer[self.buffer_bytes_sent ..];
      match send_buffer(self.socket.deref_mut(), remaining_bytes) {
        SendResult::WouldBlock(bytes_written) => {
          self.socket_writeable = false;
          self.buffer_bytes_sent += bytes_written;
          false
        },
        SendResult::Consumed => {
          self.buffer_bytes_sent += remaining_bytes.len();
          true
        },
        SendResult::IoError(_) => {
          return Some(self.total_bytes_sent);
        }
      }
    }
    else {
      false
    };

    if need_more_data {
      match self.reader.try_queue_read() {
        Ok(true) | //eof?
        Err(_) => return Some(self.total_bytes_sent),
        _ => {}
      };
    }
    
    return None;
  }

  pub fn into_parts(self) -> (Registered<Reader>, Registered<W>) {
    (self.reader, self.socket)
  }
}

impl<W: Write + AsyncSource> Handler<usize> for FileResponder<W> {
  fn handle_event(&mut self, event: &Event, _ctx: &Context) -> Option<usize> {
    //if the socket becomes readable, we don't care (http1)
    //or someone else should handle it (http2)
    //so in here we only handle reading from the file
    if event.token() == self.socket.token() && event.kind().is_writable() {
      self.socket_writeable = true;
    }
    if event.token() == self.socket.token() || event.token() == self.reader.token() {
      self.send_and_request_data()
    }
    else {
      None
    }
  }
}
