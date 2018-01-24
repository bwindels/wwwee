use io::{Handler, Context, AsyncToken};
use std::io::Write;
use buffer::Buffer;
use super::internal::ResponseBody;
use io::handlers::buffer::BufferWriter;

pub struct ResponseWriter<W> {
  headers: BufferWriter<W>,
  body: ResponseBody
}

impl<W: Write> ResponseWriter<W> {
  pub fn new(socket: W, headers: Buffer, body: ResponseBody) -> ResponseWriter<W> {
    ResponseWriter {headers: BufferWriter::new(socket, headers), body}
  }
}

impl<W: Write> Handler<()> for ResponseWriter<W> {
  fn readable(&mut self, token: AsyncToken, ctx: &Context) -> Option<()> {
    self.headers.readable(token, ctx)
  }

  fn writable(&mut self, token: AsyncToken, ctx: &Context) -> Option<()> {
    self.headers.writable(token, ctx)
  }
}
