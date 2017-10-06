use super::{
  HeaderBodySplitter,
  Request,
  BufferResponse, Buffer,
  FinishedBufferResponse
};
use mio::net::TcpStream;
use std::io::Write;

pub trait RequestHandler {
  fn read_headers(&mut self, request: &Request) -> Option<FinishedBufferResponse>;
  fn read_body(&mut self, body: &mut [u8]) -> Option<FinishedBufferResponse>;
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
          resp.set_header("Content-Type", "text/plain");
          let mut body = resp.into_body();
          write!(body, "No response from handler").unwrap();
          body.finish()
        }
      }
      else {
        let mut resp = BufferResponse::bad_request();
        resp.set_header("Content-Type", "text/plain");
        let mut body = resp.into_body();
        write!(body, "Error while parsing request").unwrap();
        body.finish()
      };
      let response_buf = response.as_slice();
      socket.write(response_buf).unwrap();
      return header_buf.len();
    }
    return 0;
  }
}