use io::{Handler, AsyncSource, Context, Registered, Event};
use io::handlers::{buffer, file};
use std::io::Write;
use buffer::Buffer;
use super::internal::ResponseBody;

enum State<W> {
  Headers(buffer::BufferWriter<W>, ResponseBody),
  FileBody(file::ResponseHandler<W>)
}

pub struct ResponseWriter<W> {
  state: Option<State<W>>
}

impl<W: Write> ResponseWriter<W> {
  pub fn new(socket: Registered<W>, headers: Buffer, body: ResponseBody) -> ResponseWriter<W> {
    ResponseWriter {
      state: Some(State::Headers(buffer::BufferWriter::new(socket, headers), body))
    }
  }

  fn next_state(&mut self, ctx: &Context) -> Option<State<W>> {
    let state = self.state.take();
    match state {
      Some(State::Headers(header_writer, ResponseBody::File(file_reader))) => {
        let socket = header_writer.into_writer();
        let file_reader = ctx.register(file_reader).unwrap();
        let file_writer = file::ResponseHandler::new(socket, file_reader);
        Some(State::FileBody(file_writer))
      },
      Some(State::FileBody(file_writer)) => {
        file_writer.into_reader().into_deregistered(ctx).unwrap();
        None
      },
      _ => None
    }
  }
}

impl<W: Write + AsyncSource> Handler<()> for ResponseWriter<W> {
  fn handle_event(&mut self, event: &Event, ctx: &Context) -> Option<()> {
    let result = match self.state {
      Some(State::Headers(ref mut header_writer, _)) => {
        header_writer.handle_event(event, ctx)
      },
      Some(State::FileBody(ref mut file_writer)) => {
        file_writer.handle_event(event, ctx)
      },
      _ => None
    };
    if result.is_some() { //a subhandler has finished, switch to the next
      self.state = self.next_state(ctx);
    }
    match self.state {
      Some(_) => None,  //in progress
      None => Some( () ) // done
    }
  }
}
