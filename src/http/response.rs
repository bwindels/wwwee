use std::io::Write;
use std::io;
use buffer::Buffer;
use io::handlers::buffer::BufferWriter;
use super::status::Status;

pub struct Response<'a> {
  buffer: Buffer<'a>
}

impl<'a> Response<'a> {
  pub fn into_handler<W: Write>(self, writer: W) -> BufferWriter<'a, W> {
    BufferWriter::new(self.buffer, writer)
  }
}

pub trait Responder<'a> {
  fn respond(&self, status: Status) -> io::Result<HeaderWriter<'a>>;
}

pub mod implementation {
  use http::status::Status;
  use std::io;
  use io::Context;

  pub struct Responder<'b, C: 'b> {
    ctx: &'b C
  }

  impl<'b, C> Responder<'b, C> {
    pub fn new(ctx: &'b C) -> Responder<'b, C> {
      Responder {ctx}
    }
  }

  impl<'a, 'b, C: Context<'a>> super::Responder<'a> for Responder<'b, C> {
    fn respond(&self, status: Status) -> io::Result<super::HeaderWriter<'a>> {
      let buffer = self.ctx.borrow_buffer(4096)
        .map_err(|_| io::Error::new(io::ErrorKind::Other, "could not borrow buffer"))?;
      let mut response = super::HeaderWriter { buffer };
      response.write_head(status.0, status.1)?;
      Ok(response)
    }
  }
  
}


pub struct HeaderWriter<'a> {
  buffer: Buffer<'a>
}

impl<'a> HeaderWriter<'a> {

  pub fn write_head(&mut self, status: u16, description: &str) -> io::Result<()> {
    write!(self.buffer, "HTTP/1.1 {} {}", status, description)
  }

  pub fn set_header(&mut self, name: &str, value: &str) -> io::Result<()> {
    write!(&mut self.buffer, "\r\n{}:{}", name, value)
  }

  pub fn into_body(mut self) -> io::Result<BodyWriter<'a>> {
    write!(&mut self.buffer, "\r\n\r\n")?;
    Ok(BodyWriter::new(self.buffer))
  }
}

pub struct BodyWriter<'a> {
  buffer: Buffer<'a>,
  len_before_body: usize
}

impl<'a> BodyWriter<'a> {
  fn new(buffer: Buffer<'a>) -> BodyWriter<'a> {
    let len_before_body = buffer.len();
    BodyWriter {buffer, len_before_body}
  }

  pub fn finish(self) -> Response<'a> {
    println!("Content-Length should be {}", self.len_before_body - self.buffer.len());
    Response {buffer: self.buffer}
  }
}

impl<'a> Write for BodyWriter<'a> {
  fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
    self.buffer.write(buf)
  }

  fn flush(&mut self) -> io::Result<()> {
    self.buffer.flush()
  }
}
