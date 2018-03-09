use std::io::{Write, Read, ErrorKind, Result};

pub fn send_buffer(socket: &mut Write, buffer: &[u8]) -> Result<IoReport> {
  let mut remaining_bytes = buffer;
  let mut bytes_written = 0usize;

  while remaining_bytes.len() != 0 {
    match socket.write(remaining_bytes) {
      Ok(bytes_written_write) => {
        bytes_written += bytes_written_write;
        remaining_bytes = &remaining_bytes[bytes_written_write ..];
      },
      Err(err) => {
        match err.kind() {
          ErrorKind::Interrupted => {}, //retry
          ErrorKind::WouldBlock =>
            return Ok(IoReport::from_count(bytes_written, remaining_bytes.len())),
          _ =>
            return Err(err)
        };
      }
    };
  }

  Ok(IoReport::from_count(bytes_written, remaining_bytes.len()))
}

pub fn receive_buffer(socket: &mut Read, mut buffer: &mut [u8]) -> Result<IoReport> {
  let mut bytes_read = 0usize;

  while buffer.len() != 0 {
    match socket.read(buffer) {
      Ok(len) => {
        bytes_read += len;
        buffer = &mut buffer[len ..];
      },
      Err(err) => {
        match err.kind() {
          ErrorKind::Interrupted => {}, //retry
          ErrorKind::WouldBlock =>
            return Ok(IoReport::from_count(bytes_read, buffer.len())),
          _ =>
            return Err(err)
        };
      }
    };
  }

  Ok(IoReport::from_count(bytes_read, buffer.len()))
}



#[derive(Clone, Copy)]
pub enum IoReport {
  Empty,
  Partial(usize),
  Complete(usize)
}

impl IoReport {

  pub fn from_count(byte_count: usize, remaining_bytes: usize) -> IoReport {
    if byte_count == 0 {
      IoReport::Empty
    }
    else if remaining_bytes != 0 {
      IoReport::Partial(byte_count)
    }
    else {
      IoReport::Complete(byte_count)
    }
  }

  pub fn is_partial(self) -> bool {
    match self {
      IoReport::Partial(_) => true,
      _ => false
    }
  }

  pub fn is_complete(self) -> bool {
    match self {
      IoReport::Complete(_) => true,
      _ => false
    }
  }

  pub fn is_empty(self) -> bool {
    self.byte_count() == 0
  }

  pub fn byte_count(self) -> usize {
    match self {
      IoReport::Complete(byte_count) => byte_count,
      IoReport::Partial(byte_count) => byte_count,
      IoReport::Empty => 0
    }
  }
}
