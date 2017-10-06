use std::io::Write;
use std::io;

pub trait Buffer<'a> {
  fn as_slice(&'a mut self) -> &'a [u8];
}

pub struct BufferResponse {
  buffer: [u8; 4096],
  write_offset: usize,
  finished_head: bool
}

impl<'a> Buffer<'a> for BufferResponse {
  fn as_slice(&'a mut self) -> &'a [u8] {
    &self.buffer[..self.write_offset]
  }
}

impl Write for BufferResponse {
  fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
    let total_len = self.buffer.len();
    let mut writer = self.buffer[self.write_offset ..].as_mut();
    let result = writer.write(buf);
    self.write_offset = total_len - writer.len();
    result
  }

  fn flush(&mut self) -> io::Result<()> {
    Ok(())
  }
}

impl BufferResponse {

  pub fn new(status: u16, description: &str) -> BufferResponse {
    let mut resp = BufferResponse {
      buffer: [0; 4096],
      write_offset: 0,
      finished_head: false
    };
    resp.write_head(status, description);
    resp
  }

  pub fn ok() -> BufferResponse {
    BufferResponse::new(200, "OK")
  }

  pub fn bad_request() -> BufferResponse {
    BufferResponse::new(400, "Bad request")
  }

  fn write_head(&mut self, status: u16, description: &str) {
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

  pub fn finish_head(&mut self) {
    write!(self, "\r\n\r\n").unwrap();
  }
  
}
/*
struct ResponseHeaders {}
struct FinishedResponse {}

fn send_response() -> FinishedResponse {
  let resp = BufferResponse::OK().set("Content-Type", "text/html");
  let content_length = resp.reserve_header("Content-Length", 10);
  let resp = resp.into_body();
  write!(resp, "hello world! {} {}", req.method(), req.uri());
  resp.finish()
}*/