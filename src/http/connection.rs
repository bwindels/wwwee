use super::{
  HeaderBodySplitter,
  Request,
  BufferResponse, Buffer
};
use mio::net::TcpStream;
use std::io::Write;

pub trait RequestHandler {
  fn read_headers(&mut self, request: &Request) -> Option<BufferResponse>;
  fn read_body(&mut self, body: &mut [u8]) -> Option<BufferResponse>;
}

pub struct ConnectionHandler<T> {
  header_body_splitter: HeaderBodySplitter,
  handler: T,
  //content_length: u64
}

impl<T> ConnectionHandler<T> {
  pub fn new(handler: T) -> ConnectionHandler<T> {
    ConnectionHandler {
      header_body_splitter: HeaderBodySplitter::new(),
      handler
    }
  }
}

impl<T: RequestHandler> ::connection::ConnectionHandler for ConnectionHandler<T> {

  //type WriteJob = ResponseJob;

  fn bytes_available(&mut self, buffer: &mut [u8], socket: &mut TcpStream) -> usize {
    if let Some((header_buf, _)) = self.header_body_splitter.try_split(buffer) {
      let mut response = if let Ok(req) = Request::parse(header_buf) {
        let mut resp = BufferResponse::new();
        resp.write_head(200, "OK");
        resp.write_header("Content-Type", "text/plain");
        resp.write_body_str(req.uri());
        resp
      }
      else {
        let mut resp = BufferResponse::new();
        resp.write_head(400, "Bad request");
        resp.write_header("Content-Type", "text/plain");
        resp.write_body_str("Error while parsing request");
        resp
      };
      let response_buf = response.as_slice();
      socket.write(response_buf).unwrap();
      return header_buf.len();
    }
    return 0;
  }
}