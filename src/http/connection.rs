use super::{
  HeaderBodySplitter,
  Request,
  BufferResponse,
  Buffer,
  FinishedBufferResponse,
  RequestError
};
use mio::net::TcpStream;
use std::io::{Write, ErrorKind};
use std::io;

pub trait RequestHandler {
  fn read_headers(&mut self, request: &Request) -> io::Result<Option<FinishedBufferResponse>>;
  fn read_body(&mut self, body: &mut [u8]) -> io::Result<Option<FinishedBufferResponse>>;
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
      let consumed_bytes = header_buf.len();
      let request = Request::parse(header_buf);

      let mut response = request.map(|req| {
        self.handler.read_headers(&req)
          .unwrap_or_else(handle_io_error)
      })
      .unwrap_or_else(handle_request_error)
      .unwrap_or_else(handle_no_response);

      let response_buf = response.as_slice();
      socket.write(response_buf).unwrap();
      return consumed_bytes;
    }
    return 0;
  }
}

#[allow(unused_must_use)]
fn handle_no_response() -> FinishedBufferResponse {
  let mut resp = BufferResponse::internal_server_error();
  resp.set_header("Content-Type", "text/plain");
  let mut body = resp.into_body();
  write!(body, "No response from handler");
  body.finish()
}

#[allow(unused_must_use)]
fn handle_io_error(err: io::Error) -> Option<FinishedBufferResponse> {
  let mut resp = BufferResponse::internal_server_error();
  let msg = match err.kind() {
    ErrorKind::UnexpectedEof => "Response too big for buffer",
    _ => "Unknown IO error"
  };
  resp.set_header("Content-Type", "text/plain");
  let mut body = resp.into_body();
  write!(body, "{}", msg);
  Some(body.finish())
}

#[allow(unused_must_use)]
fn handle_request_error(err: RequestError) -> Option<FinishedBufferResponse> {
  let mut resp = BufferResponse::bad_request();
  let msg = match err {
    RequestError::InvalidRequestLine => "Invalid request line",
    RequestError::InvalidHeader => "Invalid header",
    RequestError::InvalidEncoding => "Request not encoded with UTF8",
    RequestError::UrlEncodedNul => "URL encoded value contains NUL character"
  };
  resp.set_header("Content-Type", "text/plain");
  let mut body = resp.into_body();
  write!(body, "{}", msg);
  Some(body.finish())
}

