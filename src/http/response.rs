use std::io::Write;
use std::io;
use buffer::Buffer;
use io::{Context, Registered};
use io::sources::file;
use http::status::Status;
use super::response_writer::ResponseWriter;

pub enum ResponseBody {
  InBuffer,
  File(file::Reader)
}

pub struct Response {
  buffer: Buffer,
  body: ResponseBody
}

impl Response {
  pub fn from_buffer(response: Buffer) -> Response {
    Response {buffer: response, body: ResponseBody::InBuffer}
  }

  pub fn from_file(headers: Buffer, file: file::Reader) -> Response {
    Response {buffer: headers, body: ResponseBody::File(file)}
  }

  pub fn into_handler<W: Write>(self, writer: Registered<W>) -> ResponseWriter<W> {
    ResponseWriter::new(writer, self.buffer, self.body)
  }
}

pub struct Responder<'a> {
  ctx: &'a Context<'a>
}

impl<'a> Responder<'a> {
  pub fn new(ctx: &'a Context) -> Responder<'a> {
    Responder {ctx}
  }

  pub fn respond(&self, status: Status) -> io::Result<HeaderWriter> {
    let buffer = Buffer::new();
    let mut header_writer = HeaderWriter { buffer };
    header_writer.write_head(status.0, status.1)?;
    Ok(header_writer)
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

  pub fn set_header_usize(&mut self, name: &str, value: usize) -> io::Result<()> {
    write!(&mut self.buffer, "\r\n{}:{}", name, value)
  }

  pub fn into_body(mut self) -> io::Result<BodyWriter> {
    write!(&mut self.buffer, "\r\n\r\n")?;
    Ok(BodyWriter::new(self.buffer))
  }

  pub fn finish_with_file(mut self, file: file::Reader) -> io::Result<Option<Response>> {
    write!(&mut self.buffer, "\r\n\r\n")?;
    Ok(Some(Response::from_file(self.buffer, file)))
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
    println!("Content-Length should be {}", self.buffer.len() - self.len_before_body);
    Response::from_buffer(self.buffer)
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
