use http::{
  HeaderBodySplitter,
  Request,
  RequestError,
  Responder,
  Response,
  status,
};
use buffer::Buffer;
use io;
use io::handlers::buffer::BufferWriter;
use std;

pub trait RequestHandler {
  
  fn read_headers(&mut self, request: &Request, responder: &Responder)
    -> std::io::Result<Option<Response>>
  {
    Ok(None)
  }

  fn read_body(&mut self, body: &mut [u8], responder: &Responder)
    -> std::io::Result<Option<Response>>
  {
    Ok(None)
  }

}

pub struct Handler<T, S> {
  header_body_splitter: HeaderBodySplitter,
  handler: T,
  read_buffer: Buffer,
  socket: S
  //content_length: u64
}
/*
enum Stage {
  SearchHeaderEnd(HeaderBodySplitter),
  HeadersParsed(Request)
}
*/
impl<T, S> Handler<T, S> {

  pub fn new(handler: T, socket: S) -> Handler<T, S> {
    Handler {
      header_body_splitter: HeaderBodySplitter::new(),
      handler,
      socket,
      read_buffer: Buffer::new()
    }
  }
}

impl<T, S>
  io::Handler<Option<BufferWriter<S>>> 
for
  Handler<T, S>
where
  T: RequestHandler,
  S: std::io::Read + std::io::Write
{

  fn readable(&mut self, _token: io::AsyncToken, ctx: &io::Context) -> Option<Option<io::handlers::buffer::BufferWriter<S>>> {
    if let Ok(_) = self.read_buffer.read_from(&mut self.socket) {
      let mut read_buffer = self.read_buffer.as_mut_slice();
      if let Some((header_buf, _)) = 
        self.header_body_splitter.try_split(&mut read_buffer)
      {
        // let consumed_bytes = header_buf.len();
        let request = Request::parse(header_buf).unwrap();
        let mut response = {
          let responder = ::http::response::Responder::new(ctx);
          self.handler.read_headers(&request, &responder).unwrap().unwrap()
        };
        /*let mut response = {
            request.map(|req| {
            self.handler.read_headers(&req, &responder)
              .unwrap_or_else(|err| handle_io_error(err, &responder))
          })
          .unwrap_or_else(|err| handle_request_error(err, &responder))
          .unwrap_or_else(|| handle_no_response(&responder))
        };*/
        
        //let response_handler = response.into_handler(self.socket);
        //io::OperationState::Finished(Some(response_handler))
        None
      }
      else {
        None
      }
    }
    else {
      Some(None)
    }
  }
}
/*
#[allow(unused_must_use)]
fn handle_no_response<'a, 'b>(responder: &Responder<'b>) -> Response<'b> {
  let mut resp = responder.respond(status::INTERNAL_SERVER_ERROR);
  resp.set_header("Content-Type", "text/plain");
  let mut body = resp.into_body();
  write!(body, "No response from handler");
  body.finish()
}

#[allow(unused_must_use)]
fn handle_io_error<'a, 'b>(err: std::io::Error, responder: &Responder<'a, 'b>) -> Option<Response<'b>> {
  let mut resp = responder.respond(status::INTERNAL_SERVER_ERROR);
  let msg = match err.kind() {
    std::io::ErrorKind::WriteZero => "Response too big for buffer",
    _ => "Unknown IO error"
  };
  resp.set_header("Content-Type", "text/plain");
  let mut body = resp.into_body();
  write!(body, "{}", msg);
  Some(body.finish())
}

#[allow(unused_must_use)]
fn handle_request_error<'a, 'b>(err: RequestError, responder: &Responder<'a, 'b>) -> Option<Response<'b>> {
  let mut resp = responder.respond(status::BAD_REQUEST);
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

*/
