use http;
use http::headers::Authorization;
use std::io::Write;
use std::io;

pub struct BasicAuthHandler<'a, T, V> {
  child_handler: T,
  validate_password: V,
  realm: &'a str
}

impl<'a, T, V> BasicAuthHandler<'a, T, V>
where
  T: http::RequestHandler,
  V: Fn(&http::headers::authorization::BasicCredentials) -> bool
{
  pub fn new(child_handler: T, realm: &'a str, validate_password: V) -> BasicAuthHandler<'a, T, V> {
    BasicAuthHandler { child_handler, validate_password, realm }
  }
}

impl<'a, T, V> http::RequestHandler for BasicAuthHandler<'a, T, V>
where
  T: http::RequestHandler,
  V: Fn(&http::headers::authorization::BasicCredentials) -> bool
{
  fn read_headers(&mut self, request: &http::Request, responder: &http::Responder) -> io::Result<Option<http::Response>> {
    if let Some(Authorization::Basic(ref credentials)) = request.headers().authorization {
      if (self.validate_password)(&credentials) {
        return self.child_handler.read_headers(request, responder);
      }
    }
    
    let mut response = responder.respond(http::status::UNAUTHORIZED)?;
    response.set_header_writer("WWW-Authenticate", |ref mut value| {
      write!(value, "Basic realm=\"{}\", charset=\"UTF-8\"", self.realm)
    })?;
    Ok(Some(response.into_body()?.finish()))
  }

  fn read_body(&mut self, body: &mut [u8], responder: &http::Responder)
    -> std::io::Result<Option<http::Response>>
  {
    // if read_headers didn't respond already because UNAUTHORIZED,
    // it's safe to assume we are authorized and can just forward read_body
    self.child_handler.read_body(body, responder)
  }
}
