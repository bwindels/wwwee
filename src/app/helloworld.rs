use http::{RequestHandler, Request, BufferResponse, FinishedBufferResponse};
use std::io::Write;

pub struct HelloWorld {
  
}

impl HelloWorld {
  pub fn new() -> HelloWorld {
    HelloWorld {}
  }
}

impl RequestHandler for HelloWorld {
  fn read_headers(&mut self, req: &Request) -> Option<FinishedBufferResponse> {
    let mut resp = BufferResponse::ok();
    resp.set_header("Content-Type", "text/html");
    let mut body = resp.into_body();
    write!(body, "<h1>Hello World!</h1>").unwrap();
    write!(body, "<p>You requested: <code>{} {}</code></p>\n", req.method(), req.uri()).unwrap();
    write!(body, "<p>Query string: <code>{}</code></p>\n", req.querystring()).unwrap();
    if let Some(host) = req.headers().host {
      write!(body, "<p>With host: <code>{}</code></p>\n", host).unwrap();
    }
    Some(body.finish())
  }
  fn read_body(&mut self, _: &mut [u8]) -> Option<FinishedBufferResponse> {
    None
  }
}