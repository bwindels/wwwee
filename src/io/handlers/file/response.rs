use super::Reader;

pub struct ResponseHandler {
  reader: Reader,
  total_bytes_sent: usize,
  buffer_bytes_sent: usize,
  file_token: AsyncToken,
  socket_token: AsyncToken
}

impl ResponseHandler {

  pub fn new(reader: Reader, ) -> ResponseHandler {
    ResponseHandler {
      reader,
      file_token,
      socket_token,
      bytes_sent: 0
    }
  }

  fn send_data(&mut self) -> Option<usize> {
    let buffer = self.reader.try_get_read_bytes().unwrap();
    let remaining_bytes = &buffer[self.bytes_sent ..];
    let bytes_sent += self.socket.write(remaining_bytes).unwrap();
    //assume the socket buffer is full here?
    self.socket_writeable = false;

    self.buffer_bytes_sent += bytes_sent;
    let remaining_bytes_len_after_write = 
      buffer.len() - self.buffer_bytes_sent;
    
    if remaining_bytes_len_after_write == 0 {
      self.total_bytes_sent += buffer_bytes_sent;
      let eof = !self.try_queue_read().unwrap();
      if eof {
        return Some(self.total_bytes_sent);
      }
    }
    
    return None;
  }

  fn request_data(&mut self) -> Option<usize> {

  }
}

impl io::Handler<usize> for ResponseHandler {
  fn readable(&mut self, _token: AsyncToken, _ctx: &Context) -> Option<usize> {
    //if the socket becomes readable, we don't care (http1)
    //or someone else should handle it (http2)
    //so in here we only handle reading from the file
    if self.socket_writeable {
      return self.send_data();
    }
    else {
      return None;
    }
  }

  fn writable(&mut self, _token: AsyncToken, _ctx: &Context) -> Option<usize> {
    //can only be the socket, we don't register
    //the file reader eventfd for writeable events 
    self.socket_writeable = true;
    if self.reader.has_data_ready() {
      return self.send_data();
    }
    return None;
  }
}
