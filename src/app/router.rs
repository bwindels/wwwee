use http;
use std;
use std::io::Write;

pub struct Router<F, H, D> {
  big_file: F,
  hello_world: H,
  default: D
}

impl<F, H, D> Router<F, H, D> {
  pub fn new(default: D, hello_world: H, big_file: F) -> Router<F, H, D> {
    Router {
      default,
      hello_world,
      big_file
    }
  }
}

impl<F, H, D> http::RequestHandler for Router<F, H, D>
  where
    F: http::RequestHandler,
    H: http::RequestHandler,
    D: http::RequestHandler
{
  fn read_headers(&mut self, request: &http::Request, res: &http::Responder) -> std::io::Result<Option<http::Response>> {
    if request.url() == "/download/" {
      self.big_file.read_headers(request, res)
    }
    else if request.url().starts_with("/hello/") {
      self.hello_world.read_headers(request, res)
    }
    else if request.url() == "/" {
      self.default.read_headers(request, res)
    }
    else {
      let mut response = res
        .respond(http::status::NOT_FOUND)?;
      response.set_header("Content-Type", "text/plain")?;
      let mut body = response.into_body()?;
      write!(body, "{}", http::status::NOT_FOUND.1)?;
      Ok(Some(body.finish()))
    }
  }
}
