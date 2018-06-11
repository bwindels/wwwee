use std;
use io;
use super::context::Context;

pub struct Handler<'a, H> {
  tls_context: Context<'a>,
  child_handler: H
}

impl<'a, H> Handler<'a, H> {
  fn handle_socket_event<T>(&mut self, event: &io::Event, ctx: &mut io::Context)
    -> std::io::Result<Option<T>>
    where H: io::Handler<T>
  {
    let (socket, child_ctx_factory) = ctx.as_socket_and_factory(); 
    let mut tls_socket = self.tls_context.wrap_socket(socket);
    let event_kind = event.kind();

    if event_kind.is_readable() {
      while tls_socket.read_records()?.should_retry() {};
    }
    if event_kind.is_writable() {
      while tls_socket.write_records()?.should_retry() {};
    }

    let child_event_kind = event_kind
      .with_readable(tls_socket.is_readable())
      .with_writable(tls_socket.is_writable());

    if child_event_kind.has_any() {
      let child_event = event.with_kind(child_event_kind);
      let mut child_ctx = child_ctx_factory.into_context(&mut tls_socket);
      Ok(self.child_handler.handle_event(&child_event, &mut child_ctx))
    }
    else {
      Ok(None) //need more events
    }
  }
}

/*
here we need to
- handle gracefully when the socket isn't writable (wait for next event to send)
- check upon receiving records whether there is something to send out again (handshake!)
- check if there is plaintext available, if so, wrap the socket and forward the event to child_handler 
*/

impl<'a, T, H: io::Handler<T>> io::Handler<T> for Handler<'a, H> {
  fn handle_event(&mut self, event: &io::Event, ctx: &mut io::Context) -> Option<T> {

    if ctx.socket().is_source_of(event) {
      let result = self.handle_socket_event(event, ctx);
      //result.map_err()
      result.unwrap()
    }
    else {
      let (socket, child_ctx_factory) = ctx.as_socket_and_factory();
      let mut tls_socket = self.tls_context.wrap_socket(socket);
      let mut child_ctx = child_ctx_factory.into_context(&mut tls_socket);
      self.child_handler.handle_event(event, &mut child_ctx)
    }
  }
}
