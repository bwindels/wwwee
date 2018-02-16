use http;
use std;
use std::io::Write;

pub struct Router<F, H, I, D> {
  big_file: F,
  hello_world: H,
  image: I,
  default: D
}

impl<F, H, I, D> Router<F, H, I, D> {
  pub fn new(default: D, hello_world: H, big_file: F, image: I) -> Router<F, H, I, D> {
    Router {
      default,
      hello_world,
      big_file,
      image
    }
  }
}

impl<F, H, I, D> http::RequestHandler for Router<F, H, I, D>
  where
    F: http::RequestHandler,
    H: http::RequestHandler,
    I: http::RequestHandler,
    D: http::RequestHandler
{
  fn read_headers(&mut self, request: &http::Request, res: &http::Responder) -> std::io::Result<Option<http::Response>> {
    if request.url() == "/download/" {
      self.big_file.read_headers(request, res)
    }
    else if request.url() == "/some_image/" {
      self.image.read_headers(request, res)
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
