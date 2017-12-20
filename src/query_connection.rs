use io::{Handler, AsyncToken, Context};

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

  fn handle_event<FQ, FR>(
    &mut self,
    request_forwarder: FQ,
    response_forwarder: FR) -> Option<()>
  where
      FQ: FnOnce(&mut Q) -> Option<Option<R>>,
      FR: FnOnce(&mut R) -> Option<()>,
  {
    let (response_handler, result) = match self.stage {
      Stage::Request(ref mut handler) => {
        match request_forwarder(handler) {
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
      Stage::Response(ref mut handler) => (None, response_forwarder(handler))
    };
    if let Some(response_handler) = response_handler {
      self.stage = Stage::Response(response_handler);
    }
    result
  }
}

impl<Q: Handler<Option<R>>, R: Handler<()>> Handler<()> for QueryConnection<Q, R> {
  
  fn readable(&mut self, token: AsyncToken, ctx: &Context) -> Option<()> {
    self.handle_event(
      |request_handler| request_handler.readable(token, ctx),
      |response_handler| response_handler.readable(token, ctx),
    )
  }

  fn writable(&mut self, token: AsyncToken, ctx: &Context) -> Option<()> {
    self.handle_event(
      |request_handler| request_handler.writable(token, ctx),
      |response_handler| response_handler.writable(token, ctx),
    )
  }

}
