use std::io::{Write, Read, ErrorKind, Result};

pub fn send_buffer(socket: &mut Write, buffer: &[u8]) -> Result<IoReport> {
  let report = IoReport::with_buffer_size(buffer.len());
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
            return Ok(report.would_block(bytes_written)),
          _ =>
            return Err(err)
        };
      }
    };
  }

  Ok(report.ok(bytes_written))
}

pub fn receive_buffer(socket: &mut Read, buffer: &mut [u8]) -> Result<IoReport> {
  let report = IoReport::with_buffer_size(buffer.len());
  let mut bytes_read = 0usize;

  while bytes_read != buffer.len() {
    match socket.read(&mut buffer[bytes_read ..]) {
      Ok(0) => return Ok(report.ok(bytes_read)),
      Ok(len) => bytes_read += len,
      Err(err) => match err.kind() {
          ErrorKind::Interrupted => {}, //retry
          ErrorKind::WouldBlock => return Ok(report.would_block(bytes_read)),
          _ => return Err(err)
      }
    }
  }

  Ok(report.ok(bytes_read))
}

#[derive(Clone, Copy)]
pub struct IoReportBuilder(usize);

impl IoReportBuilder {

  pub fn would_block(self, byte_count: usize) -> IoReport {
    IoReport {
      would_block: true,
      buffer_size: self.0,
      byte_count
    }
  }

  pub fn ok(self, byte_count: usize) -> IoReport {
    IoReport {
      would_block: false,
      buffer_size: self.0,
      byte_count
    }
  }
}

/// Decribes the result of a single, non-blocking IO operation
#[derive(Clone, Copy)]
pub struct IoReport {
  would_block: bool,
  byte_count: usize,
  buffer_size: usize
}

impl IoReport {

  pub fn with_buffer_size(buffer_size: usize) -> IoReportBuilder {
    IoReportBuilder(buffer_size)
  }

  pub fn is_partial(&self) -> bool {
    !self.is_empty() && !self.is_complete()
  }

  pub fn would_block(&self) -> bool {
    self.would_block
  }

  pub fn should_retry(&self) -> bool {
    !self.is_empty() && !self.would_block()
  }

  pub fn is_complete(&self) -> bool {
    !self.is_empty() && self.byte_count == self.buffer_size
  }

  pub fn is_empty(&self) -> bool {
    self.byte_count == 0
  }

  pub fn byte_count(&self) -> usize {
    self.byte_count
  }
}
