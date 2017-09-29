use std::path::Path;

struct FileBody<'a> {
  path: &'a Path,
  offset: usize,
  length: Option<usize>
}

enum ResponseBody<'a> {
  Bytes(&'a [u8]),
  File(FileBody<'a>)
}

struct Response<'a> {
  headers: &'a [u8],
  body: Option<ResponseBody<'a>>
}

struct Header<'a> {
  pub name: &'a str,
  pub value: &'a str
}

struct HeaderIterator<'a> {
  headers: &'a str
}

impl<'a> Iterator for HeaderIterator<'a> {
  type Item = Header<'a>;

  fn next(&mut self) -> Option<Self::Item> {
    None
  }
}

trait Handler {
  fn bytes_available(&mut self, bytes: &mut [u8], stream: &TcpStream) -> usize;
}

trait HttpHandler<T> {
  fn read_headers(&mut self, http_version: &str, method: &str, uri: &str, headers: HeaderIterator) -> Option<Response>;
  fn read_body(&mut self, body: T) -> Response;
}