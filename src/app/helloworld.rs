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
    write!(body, "<!DOCTYPE html><html><head><meta charset=\"utf-8\"/></head><body>").unwrap();
    write!(body, "<h1>Hello World!</h1>").unwrap();
    write!(body, "<p>You requested: <code>{} {}</code></p>", req.method(), req.url()).unwrap();
    
    write!(body, "<p>Query parameters:</p>").unwrap();
    write!(body, "<ul>").unwrap();
    for p in req.query_params() {
      write!(body, "<li><code>\"{}\"</code> = <code>\"{}\"</code></li>", p.name, p.value).unwrap();
    }
    write!(body, "</ul>").unwrap();
    if let Some(host) = req.headers().host {
      write!(body, "<p>With host: <code>{}</code></p>\n", host).unwrap();
    }
    write!(body, "</body></html>").unwrap();
    Some(body.finish())
  }
  fn read_body(&mut self, _: &mut [u8]) -> Option<FinishedBufferResponse> {
    None
  }
}