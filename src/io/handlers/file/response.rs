use super::Reader;
use io::{AsyncToken, Handler, Registered, Context};
use io::handlers::{send_buffer, SendResult};
use std::io::Write;

// TODO: move socket to Registered<W> as well, so we don't need to keep track of token seperately
pub struct ResponseHandler<W> {
  reader: Registered<Reader>,
  socket: W,
  socket_token: AsyncToken,
  total_bytes_sent: usize,
  buffer_bytes_sent: usize,
  socket_writeable: bool
}

impl<W: Write> ResponseHandler<W> {

  pub fn new(socket: W, socket_token: AsyncToken, reader: Registered<Reader>) -> ResponseHandler<W> {
    ResponseHandler {
      reader,
      socket,
      socket_token,
      total_bytes_sent: 0,
      buffer_bytes_sent: 0,
      socket_writeable: false
    }
  }

  fn send_and_request_data(&mut self) -> Option<usize> {
    let need_more_data = if let Ok(buffer) = self.reader.try_get_read_bytes() {
      let mut remaining_bytes = &buffer[self.buffer_bytes_sent ..];
      match send_buffer(&mut self.socket, remaining_bytes) {
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

  pub fn into_reader(self) -> Registered<Reader> {
    self.reader
  }
}

impl<W: Write> Handler<usize> for ResponseHandler<W> {
  fn readable(&mut self, token: AsyncToken, _ctx: &Context) -> Option<usize> {
    //if the socket becomes readable, we don't care (http1)
    //or someone else should handle it (http2)
    //so in here we only handle reading from the file
    if token == self.reader.token() && self.socket_writeable {
      self.send_and_request_data()
    }
    else {
      None  //wait first for socket to become writeable
    }
  }

  fn writable(&mut self, _token: AsyncToken, _ctx: &Context) -> Option<usize> {
    //can only be the socket, we don't register
    //the file reader eventfd for writeable events 
    self.socket_writeable = true;
    self.send_and_request_data()
  }
}
