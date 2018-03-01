use http::{
  Request,
  Responder,
  Response,
  RequestError,
  status
};
use http::internal::*;
use buffer::Buffer;
use io;
use std;
use std::io::Write;
use io::ReadDst;

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

pub struct Handler<T> {
  header_body_splitter: HeaderBodySplitter,
  handler: T,
  read_buffer: Buffer,
}

impl<T> Handler<T> {

  pub fn new(handler: T) -> Handler<T> {
    Handler {
      header_body_splitter: HeaderBodySplitter::new(),
      handler,
      read_buffer: Buffer::new()
    }
  }
}

impl<T: RequestHandler> io::Handler<Option<ResponseWriter>> for Handler<T>
{

  fn handle_event(&mut self, event: &io::Event, ctx: &io::Context) -> Option<Option<ResponseWriter>>
  {
    let socket = ctx.socket();

    if !event.kind().is_readable() {
      return None;
    }

    match self.read_buffer.read_from(&mut socket) {
      Err(err) => {
        println!("dropping request because error while reading socket: {:?}", err.kind());
        Some(None)   //drop request
      },
      Ok(0) => {
        Some(None)   //drop request
      },
      Ok(_) => {
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
          let response_handler = response.into_handler();
          Some(Some(response_handler))
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
  Response::from_buffer(ResponseMetaInfo::from_status(500), buffer)
}

fn handle_io_error(err: std::io::Error, responder: &Responder) -> Option<Response> {
  let mut resp = responder.respond(status::INTERNAL_SERVER_ERROR).ok()?;
  /*let msg = match err.kind() {
    std::io::ErrorKind::WriteZero => "Response too big for buffer",
    _ => "Unknown IO error:"
  };*/
  resp.set_header("Content-Type", "text/plain").ok()?;
  let mut body = resp.into_body().ok()?;
  write!(body, "io error: {:?}", err).ok()?;
  Some(body.finish())
}

fn handle_request_error(err: RequestError, responder: &Responder) -> Option<Response> {
  let mut resp = responder.respond(status::BAD_REQUEST).ok()?;
  let msg = match err {
    RequestError::InvalidRequestLine => "Invalid request line",
    RequestError::InvalidHeader => "Invalid header",
    RequestError::InvalidEncoding => "Request not encoded with UTF8",
    RequestError::UrlEncodedNul => "URL encoded value contains NUL character"
  };
  resp.set_header("Content-Type", "text/plain").ok()?;
  let mut body = resp.into_body().ok()?;
  write!(body, "{}", msg).ok()?;
  Some(body.finish())
}
