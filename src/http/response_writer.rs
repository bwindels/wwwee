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

  fn process_event_result(&mut self, result: Option<usize>, ctx: &Context) -> Option<()> {
    if result.is_some() {
      let state = self.state.take();
      let (new_state, result) = match state {
        Some(State::Headers(header_writer, ResponseBody::File(file_reader))) => {
          let socket = header_writer.into_writer();
          let file_reader = ctx.register(file_reader).unwrap();
          let file_writer = file::ResponseHandler::new(socket, file_reader);
          (Some(State::FileBody(file_writer)), None)
        },
        Some(State::FileBody(file_writer)) => {
          file_writer.into_reader().into_deregistered(ctx).unwrap();
          (None, Some( () ))
        },
        _ => (None, Some( () ))
      };
      self.state = new_state;
      return result;
    }
    else {
      return None;
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
      None => Some(0)
    };
    self.process_event_result(result, ctx)
  }
}
