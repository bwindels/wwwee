use super::headers::{HeaderBodySplitter, HeaderIterator};
use super::Request;

pub struct Response {

}

pub trait RequestHandler<T> {
  fn read_headers(&mut self, request: &Request) -> Option<Response>;
  fn read_body(&mut self, body: T) -> Response;
}


pub struct ConnectionHandler<T> {
  header_body_splitter: HeaderBodySplitter,
  handler: T
}

impl<T> connection::ConnectionHandler for ConnectionHandler<T> {

}