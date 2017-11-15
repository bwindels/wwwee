use std::io::Write;
use std::io;

pub trait Buffer<'a> {
  fn as_slice(&'a mut self) -> &'a [u8];
}

struct ResponseBuffer {
  buffer: [u8; 4096],
  write_offset: usize,
}

impl ResponseBuffer {
  fn new() -> ResponseBuffer {
    ResponseBuffer {
      buffer: [0; 4096],
      write_offset: 0
    }
  }

  fn len(&self) -> usize {
    self.buffer.len() - self.write_offset
  }
}

impl<'a> Buffer<'a> for ResponseBuffer {
  fn as_slice(&'a mut self) -> &'a [u8] {
    &self.buffer[..self.write_offset]
  }
}

impl Write for ResponseBuffer {
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

pub struct BufferResponse {
  buffer: ResponseBuffer
}

fn write_head(buffer: &mut ResponseBuffer, status: u16, description: &str) {
  write!(buffer, "HTTP/1.1 {} {}", status, description).unwrap();
}

impl BufferResponse {

  pub fn new(status: u16, description: &str) -> BufferResponse {
    let mut buffer = ResponseBuffer::new();
    write_head(&mut buffer, status, description);
    BufferResponse {buffer}
  }

  pub fn ok() -> BufferResponse {
    BufferResponse::new(200, "OK")
  }

  pub fn bad_request() -> BufferResponse {
    BufferResponse::new(400, "Bad request")
  }

  pub fn internal_server_error() -> BufferResponse {
    BufferResponse::new(500, "Internal server error")
  }

  pub fn set_header(&mut self, name: &str, value: &str) {
    write!(&mut self.buffer, "\r\n{}:{}", name, value).unwrap();
  }

  pub fn into_body(mut self) -> Body {
    write!(&mut self.buffer, "\r\n\r\n").unwrap();
    Body::new(self.buffer)
  }
}

pub struct Body {
  buffer: ResponseBuffer,
  len_before_body: usize
}

impl Body {
  fn new(buffer: ResponseBuffer) -> Body {
    let len_before_body = buffer.len();
    Body {buffer, len_before_body}
  }

  pub fn finish(self) -> FinishedBufferResponse {
    println!("Content-Length should be {}", self.len_before_body - self.buffer.len());
    FinishedBufferResponse::new(self.buffer)
  }
}

impl Write for Body {
  fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
    self.buffer.write(buf)
  }

  fn flush(&mut self) -> io::Result<()> {
    self.buffer.flush()
  }
}

pub struct FinishedBufferResponse {
  buffer: ResponseBuffer
}

impl FinishedBufferResponse {
  fn new(buffer: ResponseBuffer) -> FinishedBufferResponse {
    FinishedBufferResponse {buffer}
  }
}

impl<'a> Buffer<'a> for FinishedBufferResponse {
  fn as_slice(&'a mut self) -> &'a [u8] {
    self.buffer.as_slice()
  }
}
