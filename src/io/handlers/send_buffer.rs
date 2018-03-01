use std::io::{Write, ErrorKind, Error};

pub fn send_buffer(socket: &mut Write, buffer: &[u8]) -> SendResult {
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
            return SendResult::WouldBlock(bytes_written),
          _ =>
            return SendResult::IoError(err)
        };
      }
    };
  }

  SendResult::Consumed
}

pub enum SendResult {
  Consumed,
  WouldBlock(usize),
  IoError(Error)
}
