use std::io::{Write, ErrorKind, Result};

pub fn send_buffer(socket: &mut Write, buffer: &[u8]) -> Result<SendResult> {
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
            return Ok(SendResult::Partial(bytes_written)),
          _ =>
            return Err(err)
        };
      }
    };
  }

  Ok(SendResult::Complete(bytes_written))
}

#[derive(Clone, Copy)]
pub enum SendResult {
  Partial(usize),
  Complete(usize)
}

impl SendResult {
  pub fn wrote_partial(self) -> bool {
    match self {
      SendResult::Partial(_) => true,
      _ => false
    }
  }

  pub fn wrote_complete(self) -> bool {
    match self {
      SendResult::Complete(_) => true,
      _ => false
    }
  }

  pub fn bytes_written(self) -> usize {
    match self {
      SendResult::Complete(bytes_written) => bytes_written,
      SendResult::Partial(bytes_written) => bytes_written
    }
  }
}
