use http;
use std::io::Write;
use std::io;

pub struct HelloWorld {
  
}

impl HelloWorld {
  pub fn new() -> HelloWorld {
    HelloWorld {}
  }
}

impl http::RequestHandler for HelloWorld {
  fn read_headers(&mut self, req: &http::Request, responder: &http::Responder) -> io::Result<Option<http::Response>> {
    let mut resp = responder.respond(http::status::OK)?;
    resp.set_header("Content-Type", "text/html")?;
    let mut body = resp.into_body()?;
    write!(body, "<!DOCTYPE html><html><head><meta charset=\"utf-8\"/></head><body>")?;
    write!(body, "<h1>Hello World!</h1>")?;
    write!(body, "<p>You requested: <code>{} {}</code></p>", req.method(), req.url())?;
    write!(body, "<p>Query parameters:</p>")?;
    write!(body, "<ul>")?;
    for p in req.query_params() {
      write!(body, "<li><code>\"{}\"</code> = <code>\"{}\"</code></li>", p.name, p.value)?;
    }
    write!(body, "</ul>")?;
    if let Some(host) = req.headers().host {
      write!(body, "<p>With host: <code>{}</code></p>\n", host)?;
    }
    write!(body, "</body></html>")?;
    Ok(Some(body.finish()))
  }
}
