use io::{Handler, OperationState, AsyncToken, Context};

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

impl<Q: Handler<R>, R: Handler<()>> QueryConnection<Q, R> {
  pub fn new(request_handler: Q) -> QueryConnection<Q, R> {
    QueryConnection {stage: Stage::Request(request_handler)}
  }

  fn handle_event<FQ, FR>(
    &mut self,
    request_forwarder: FQ,
    response_forwarder: FR) -> OperationState<()>
  where
      FQ: FnOnce(&mut Handler<R>) -> OperationState<R>,
      FR: FnOnce(&mut Handler<()>) -> OperationState<()>,
  {
    let (response_handler, result) = match self.stage {
      Stage::Request(ref mut handler) => {
        (
          request_forwarder(handler).into_option(),
          OperationState::InProgress
        )
      },
      Stage::Response(ref mut handler) => (None, response_forwarder(handler))
    };
    if let Some(response_handler) = response_handler {
      self.stage = Stage::Response(response_handler);
    }
    result
  }
}

impl<Q: Handler<R>, R: Handler<()>> Handler<()> for QueryConnection<Q, R> {
  
  fn readable(&mut self, token: AsyncToken, ctx: &Context) -> OperationState<()> {
    self.handle_event(
      |request_handler| request_handler.readable(token, ctx),
      |response_handler| response_handler.readable(token, ctx),
    )
  }

  fn writable(&mut self, token: AsyncToken, ctx: &Context) -> OperationState<()> {
    self.handle_event(
      |request_handler| request_handler.writable(token, ctx),
      |response_handler| response_handler.writable(token, ctx),
    )
  }

}
