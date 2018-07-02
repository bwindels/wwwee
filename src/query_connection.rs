use io::{Handler, Event, Context};

enum Stage<Q, R> {
  Request(Q),
  Response(R)
}

// an io handler that implements
// a query model, which first read a request,
// and then write a response.
pub struct QueryConnection<Q, R> {
  stage: Stage<Q, R>
}

impl<Q, R> QueryConnection<Q, R> {
  pub fn new(request_handler: Q) -> QueryConnection<Q, R> {
    QueryConnection {stage: Stage::Request(request_handler)}
  }
}

impl<Q: Handler<Option<R>>, R: Handler<()>> Handler<()> for QueryConnection<Q, R> {
  
  fn handle_event(&mut self, event: &Event, ctx: &mut Context) -> Option<()> {
    let (response_handler, mut result) = match self.stage {
      Stage::Request(ref mut handler) => {
        match handler.handle_event(event, ctx) {
          None => {
            (None, None)
          },
          Some(None) => {
            (None, Some( () ))
          },
          Some(Some(response_handler)) => {
            (Some(response_handler), None)
          },
        }
      },
      Stage::Response(ref mut handler) =>
        (None, handler.handle_event(event, ctx))
    };
    if let Some(response_handler) = response_handler {
      self.stage = Stage::Response(response_handler);
      // if the socket is writable, try responding straight away,
      // we might not get another event for this
      if event.kind().is_writable() {
        result = self.handle_event(event, ctx);
      }
    }
    result
  }
}
