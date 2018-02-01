use io::{Handler, AsyncSource, Context, Registered, Event};
use io::handlers::{BufferResponder, FileResponder};
use std::io::Write;
use buffer::Buffer;
use super::internal::ResponseBody;

enum State<W> {
  Headers(BufferResponder<W>, ResponseBody),
  FileBody(FileResponder<W>)
}

pub struct ResponseWriter<W> {
  state: Option<State<W>>
}

impl<W: Write + AsyncSource> ResponseWriter<W> {
  pub fn new(socket: Registered<W>, headers: Buffer, body: ResponseBody) -> ResponseWriter<W> {
    ResponseWriter {
      state: Some(State::Headers(BufferResponder::new(socket, headers), body))
    }
  }

  fn next_state(&mut self, ctx: &Context) -> Option<State<W>> {
    let state = self.state.take();
    let (new_state, socket_to_close) = match state {
      Some(State::Headers(header_writer, ResponseBody::File(file_reader))) => {
        let socket = header_writer.into_writer();
        let file_reader = ctx.register(file_reader).unwrap();
        let mut file_writer = FileResponder::start(socket, file_reader).unwrap();
        (Some(State::FileBody(file_writer)), None)
      },
      Some(State::Headers(header_writer, ResponseBody::InBuffer)) => {
        let socket = header_writer.into_writer();
        (None, Some(socket))
      },
      Some(State::FileBody(file_writer)) => {
        let (reader, socket) = file_writer.into_parts();
        reader.into_deregistered(ctx).unwrap();
        (None, Some(socket))
      },
      None => (None, None)
    };
    if let Some(socket) = socket_to_close {
      socket.into_deregistered(ctx).unwrap();
    }
    new_state
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
