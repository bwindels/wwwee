use std::io::Write;
use std::io;
use buffer::Buffer;
use io::Context;
use io::handlers::buffer::BufferWriter;
use http::status::Status;

pub struct Response {
  buffer: Buffer
}

impl Response {
  pub fn into_handler<W: Write>(self, writer: W) -> BufferWriter<W> {
    BufferWriter::new(self.buffer, writer)
  }
}

pub struct Responder<'a> {
  ctx: &'a Context<'a>
}

impl<'a> Responder<'a> {
  pub fn new(ctx: &'a Context) -> Responder<'a> {
    Responder {ctx}
  }

  pub fn respond(&self, status: Status) -> io::Result<super::HeaderWriter> {
    let buffer = Buffer::new();
    let mut response = super::HeaderWriter { buffer };
    response.write_head(status.0, status.1)?;
    Ok(response)
  }
}

pub struct HeaderWriter {
  buffer: Buffer
}

impl HeaderWriter {

  pub fn write_head(&mut self, status: u16, description: &str) -> io::Result<()> {
    write!(self.buffer, "HTTP/1.1 {} {}", status, description)
  }

  pub fn set_header(&mut self, name: &str, value: &str) -> io::Result<()> {
    write!(&mut self.buffer, "\r\n{}:{}", name, value)
  }

  pub fn into_body(mut self) -> io::Result<BodyWriter> {
    write!(&mut self.buffer, "\r\n\r\n")?;
    Ok(BodyWriter::new(self.buffer))
  }
}

pub struct BodyWriter {
  buffer: Buffer,
  len_before_body: usize
}

impl BodyWriter {
  fn new(buffer: Buffer) -> BodyWriter {
    let len_before_body = buffer.len();
    BodyWriter {buffer, len_before_body}
  }

  pub fn finish(self) -> Response {
    println!("Content-Length should be {}", self.len_before_body - self.buffer.len());
    Response {buffer: self.buffer}
  }
}

impl Write for BodyWriter {
  fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
    self.buffer.write(buf)
  }

  fn flush(&mut self) -> io::Result<()> {
    self.buffer.flush()
  }
}
