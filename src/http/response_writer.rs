use io::{Handler, Context, Event};
use io::handlers::{BufferResponder, FileResponder};
use buffer::Buffer;
use super::internal::ResponseBody;

enum State {
  Headers(BufferResponder, ResponseBody),
  FileBody(FileResponder)
}

pub struct ResponseWriter {
  state: Option<State>
}

impl ResponseWriter {
  pub fn new(headers: Buffer, body: ResponseBody) -> ResponseWriter {
    ResponseWriter {
      state: Some(State::Headers(BufferResponder::new(headers), body))
    }
  }

  fn next_state(&mut self, ctx: &mut Context) -> Option<State> {
    let state = self.state.take();
    let new_state = match state {
      Some(State::Headers(_, ResponseBody::File(file_reader))) => {
        let file_reader = ctx.register(file_reader).unwrap();
        let mut file_writer = FileResponder::start(file_reader).unwrap();
        Some(State::FileBody(file_writer))
      },
      Some(State::FileBody(file_writer)) => {
        let reader = file_writer.into_reader();
        reader.into_deregistered(ctx).unwrap();
        None
      },
      _ => None
    };
    new_state
  }
}

impl Handler<()> for ResponseWriter {
  fn handle_event(&mut self, event: &Event, ctx: &mut Context) -> Option<()> {
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
