use http::{RequestHandler, Request, BufferResponse};
use std::io::Write;

pub struct HelloWorld {
  
}

impl HelloWorld {
  pub fn new() -> HelloWorld {
    HelloWorld {}
  }
}

impl RequestHandler for HelloWorld {
  fn read_headers(&mut self, req: &Request) -> Option<BufferResponse> {
    let mut resp = BufferResponse::ok();
    resp.write_header("Content-Type", "text/html");
    resp.finish_head();
    write!(resp, "<h1>Hello World!</h1>").unwrap();
    write!(resp, "<p>You requested: <code>{} {}</code></p>\n", req.method(), req.uri()).unwrap();
    if let Some(host) = req.headers().host {
      write!(resp, "<p>With host: <code>{}</code></p>\n", host).unwrap();
    }
    Some(resp)
  }
  fn read_body(&mut self, body: &mut [u8]) -> Option<BufferResponse> {
    None
  }
}