use http::{
  HeaderBodySplitter,
  Request,
  Responder,
  Response,
  RequestError,
  status
};
use buffer::Buffer;
use io;
use io::handlers::buffer::BufferWriter;
use std;
use std::io::Write;

pub trait RequestHandler {
  
  fn read_headers(&mut self, _request: &Request, _responder: &Responder)
    -> std::io::Result<Option<Response>>
  {
    Ok(None)
  }

  fn read_body(&mut self, _body: &mut [u8], _responder: &Responder)
    -> std::io::Result<Option<Response>>
  {
    Ok(None)
  }

}

pub struct Handler<T, S> {
  header_body_splitter: HeaderBodySplitter,
  handler: T,
  read_buffer: Buffer,
  socket: Option<S>
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
      socket: Some(socket),
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

  fn readable(&mut self, _token: io::AsyncToken, ctx: &io::Context) -> Option<Option<io::handlers::buffer::BufferWriter<S>>>
  {
    let bytes_read = {
      let read_buffer = &mut self.read_buffer;
      self.socket.as_mut().map(|socket| {
        read_buffer.read_from(socket)
      })
    };

    match bytes_read {
      Some(Err(err)) => {
        println!("dropping request because error while reading socket: {:?}", err.kind());
        Some(None)   //drop request
      },
      None |
      Some(Ok(0)) => {
        Some(None)   //drop request
      },
      _ => {
        let mut read_buffer = self.read_buffer.as_mut_slice();
        if let Some((header_buf, _)) = 
          self.header_body_splitter.try_split(&mut read_buffer)
        {
          let request = Request::parse(header_buf);
          let response = {
            let responder = ::http::response::Responder::new(ctx);
            {
              match request {
                Ok(req) => {
                  self.handler.read_headers(&req, &responder)
                    .unwrap_or_else(|err| handle_io_error(err, &responder))
                },
                Err(err) => handle_request_error(err, &responder)
              }
            }
          };
          let response = response.unwrap_or_else(|| handle_no_response());
          
          if let Some(socket) = self.socket.take() {
            let response_handler = response.into_handler(socket);
            Some(Some(response_handler))
          }
          else {
            println!("dropping request because could not take socket out of request handler");
            Some(None) //drop request
          }
        }
        else {
          None  //in progress
        }
      }
    }
  }
}

#[allow(unused_must_use)]
fn handle_no_response() -> Response {
  let response = b"HTTP/1.1 500\r\n\r\nNo response";
  let mut buffer = Buffer::new();
  buffer.write(response);
  Response::new(buffer)
}

#[allow(unused_must_use)]
fn handle_io_error(err: std::io::Error, responder: &Responder) -> Option<Response> {
  let mut resp = responder.respond(status::INTERNAL_SERVER_ERROR).ok()?;
  let msg = match err.kind() {
    std::io::ErrorKind::WriteZero => "Response too big for buffer",
    _ => "Unknown IO error"
  };
  resp.set_header("Content-Type", "text/plain").ok()?;
  let mut body = resp.into_body().ok()?;
  write!(body, "{}", msg);
  Some(body.finish())
}

#[allow(unused_must_use)]
fn handle_request_error(err: RequestError, responder: &Responder) -> Option<Response> {
  let mut resp = responder.respond(status::BAD_REQUEST).ok()?;
  let msg = match err {
    RequestError::InvalidRequestLine => "Invalid request line",
    RequestError::InvalidHeader => "Invalid header",
    RequestError::InvalidEncoding => "Request not encoded with UTF8",
    RequestError::UrlEncodedNul => "URL encoded value contains NUL character"
  };
  resp.set_header("Content-Type", "text/plain");
  let mut body = resp.into_body().ok()?;
  write!(body, "{}", msg);
  Some(body.finish())
}
