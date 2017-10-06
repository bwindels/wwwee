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
        if let Some(response) = self.handler.read_headers(&req) {
          response
        }
        else {
          let mut resp = BufferResponse::new(500, "Internal Server Error");
          resp.write_header("Content-Type", "text/plain");
          resp.finish_head();
          write!(resp, "No response from handler").unwrap();
          resp
        }
      }
      else {
        let mut resp = BufferResponse::bad_request();
        resp.write_header("Content-Type", "text/plain");
        resp.finish_head();
        write!(resp, "Error while parsing request").unwrap();
        resp
      };
      let response_buf = response.as_slice();
      socket.write(response_buf).unwrap();
      return header_buf.len();
    }
    return 0;
  }
}