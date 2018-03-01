use std::io::Write;
use std::io;
use buffer::Buffer;
use io::Context;
use io::sources::file;
use http::status::Status;
use super::response_writer::ResponseWriter;

pub enum ResponseBody {
  InBuffer,
  File(file::Reader)
}

pub struct Response {
  meta: ResponseMetaInfo,
  buffer: Buffer,
  body: ResponseBody
}

pub struct ResponseMetaInfo {
  pub status: u16
}

impl ResponseMetaInfo {
  pub fn from_status(status: u16) -> ResponseMetaInfo {
    ResponseMetaInfo { status }
  }
}

impl Response {
  pub fn from_buffer(meta: ResponseMetaInfo, response: Buffer) -> Response {
    Response {meta, buffer: response, body: ResponseBody::InBuffer}
  }

  pub fn from_file(meta: ResponseMetaInfo, headers: Buffer, file: file::Reader) -> Response {
    Response {meta, buffer: headers, body: ResponseBody::File(file)}
  }

  pub fn into_handler(self) -> ResponseWriter {
    ResponseWriter::new(self.buffer, self.body)
  }

  pub fn status_code(&self) -> u16 {
    self.meta.status
  }
}

pub struct Responder {
}

impl Responder {
  pub fn new() -> Responder {
    Responder {}
  }

  pub fn respond(&self, status: Status) -> io::Result<HeaderWriter> {
    let mut buffer = Buffer::new();
    write_head(&mut buffer, status.0, status.1)?;
    let meta = ResponseMetaInfo { status: status.0 };
    Ok(HeaderWriter { meta, buffer })
  }
}

fn write_head(buffer: &mut Buffer, status: u16, description: &str) -> io::Result<()> {
  write!(buffer, "HTTP/1.1 {} {}", status, description)
}

pub struct HeaderWriter {
  buffer: Buffer,
  meta: ResponseMetaInfo
}

impl HeaderWriter {

  pub fn set_header(&mut self, name: &str, value: &str) -> io::Result<()> {
    // TODO: escape \r and \n
    write!(&mut self.buffer, "\r\n{}:{}", name, value)
  }

  pub fn set_header_usize(&mut self, name: &str, value: usize) -> io::Result<()> {
    write!(&mut self.buffer, "\r\n{}:{}", name, value)
  }

  pub fn set_header_writer<F>(&mut self, name: &str, callback: F)
    -> io::Result<()>
    where
      F: FnOnce(&mut HeaderValueWriter) -> io::Result<()>
  {
    write!(&mut self.buffer, "\r\n{}:", name)?;
    let mut value_writer = HeaderValueWriter { buffer: &mut self.buffer };
    callback(&mut value_writer)
  }

  pub fn into_body(mut self) -> io::Result<BodyWriter> {
    write!(&mut self.buffer, "\r\n\r\n")?;
    Ok(BodyWriter::new(self.meta, self.buffer))
  }

  pub fn finish_with_file(mut self, file: file::Reader) -> io::Result<Option<Response>> {
    write!(&mut self.buffer, "\r\n\r\n")?;
    Ok(Some(Response::from_file(self.meta, self.buffer, file)))
  }
}

pub struct BodyWriter {
  buffer: Buffer,
  meta: ResponseMetaInfo,
  len_before_body: usize
}

impl BodyWriter {
  fn new(meta: ResponseMetaInfo, buffer: Buffer) -> BodyWriter {
    let len_before_body = buffer.len();
    BodyWriter {meta, buffer, len_before_body}
  }

  pub fn finish(self) -> Response {
    //println!("Content-Length should be {}", self.buffer.len() - self.len_before_body);
    Response::from_buffer(self.meta, self.buffer)
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

pub struct HeaderValueWriter<'a> {
  buffer: &'a mut Buffer
}

impl<'a> Write for HeaderValueWriter<'a> {
  // TODO: escape \r and \n
  fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
    self.buffer.write(buf)
  }

  fn flush(&mut self) -> io::Result<()> {
    self.buffer.flush()
  }
}
