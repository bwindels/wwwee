use std::fmt;
use std::io::Write;

pub trait Buffer<'a> {
  fn as_slice(&'a mut self) -> &'a [u8];
}

pub struct BufferResponse {
  buffer: [u8; 4096],
  write_offset: usize
}

impl<'a> Buffer<'a> for BufferResponse {
  fn as_slice(&'a mut self) -> &'a [u8] {
    &self.buffer[..self.write_offset]
  }
}

impl BufferResponse {

  pub fn new() -> BufferResponse {
    BufferResponse {
      buffer: [0; 4096],
      write_offset: 0
    }
  }

  pub fn write_head(&mut self, status: u16, description: &str) {
    let total_len = self.buffer.len();
    let mut writer = self.buffer[self.write_offset ..].as_mut();
    write!(writer, "HTTP/1.1 {} {}", status, description).unwrap();
    self.write_offset = total_len - writer.len();
  }

  pub fn write_header(&mut self, name: &str, value: &str) {
    let total_len = self.buffer.len();
    let mut writer = self.buffer[self.write_offset ..].as_mut();
    write!(writer, "\r\n{}:{}", name, value).unwrap();
    self.write_offset = total_len - writer.len();
  }

  pub fn write_body_str(&mut self, body: &str) {
    let total_len = self.buffer.len();
    let mut writer = self.buffer[self.write_offset ..].as_mut();    write!(writer, "\r\n\r\n{}", body).unwrap();
    self.write_offset = total_len - writer.len();
  }
}
