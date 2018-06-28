use std;
use std::io::Write;
use io;
use super::context::Context;
use super::socket::SocketWrapper;

pub struct Handler<'a, H> {
  tls_context: Context<'a>,
  child_handler: H,
  is_closing: bool
}

impl<'a, H> Handler<'a, H> {
  pub fn new(tls_context: Context<'a>, child_handler: H) -> Handler<'a, H> {
    Handler {tls_context, child_handler, is_closing: false}
  }

  fn handle_socket_event(&mut self, event: &io::Event, ctx: &mut io::Context)
    -> std::io::Result<Option<()>>
    where H: io::Handler<()>
  {
    println!("handling socket event in TLS handler");
    let (socket, child_ctx_factory) = ctx.as_socket_and_factory(); 
    let mut tls_socket = self.tls_context.wrap_socket(socket);
    let event_kind = event.kind();

    if event_kind.is_readable() {
      while tls_socket.read_records()?.should_retry() {};
    }
    if event_kind.is_writable() {
      while tls_socket.write_records()?.should_retry() {};
    }

    println!("done with pre-child I/O");

    if self.is_closing {
      tls_socket.discard_incoming_data()?;
      if tls_socket.is_closed() {
        return Ok( Some( () ) );
      }
      else {
        return Ok( None );
      }
    }
    else {
      let child_event_kind = event_kind
        .with_readable(tls_socket.is_readable())
        .with_writable(tls_socket.is_writable());
      println!("after handshake attempt, tls socket is readable={}, writable={}, child_event_kind.has_any()={}, child_event_kind.0={}", tls_socket.is_readable(), tls_socket.is_writable(), child_event_kind.has_any(), child_event_kind.0);

      if child_event_kind.has_any() {
        tls_socket.debug_state("forwarding socket event to child handler");
        let child_event = event.with_kind(child_event_kind);
        let result = {
          let mut child_ctx = child_ctx_factory.into_context(&mut tls_socket);
          self.child_handler.handle_event(&child_event, &mut child_ctx)
        };
        // result is known, closing socket
        if result.is_some() {
          println!("closing tls socket");
          // if this can't finish now and needs to wait until next event
          // we need to store result and return that once we managed
          // to write out all the remaining records
          self.is_closing = true;
          tls_socket.flush()?;
          tls_socket.close()?;
        }
        Ok(result)
      }
      else {
        println!("waiting for more events before forwarding to child handler");
        Ok(None) //need more events
      }
    }
  }
}

/*
here we need to
- handle gracefully when the socket isn't writable (wait for next event to send)
- check upon receiving records whether there is something to send out again (handshake!)
- check if there is plaintext available, if so, wrap the socket and forward the event to child_handler 
*/

impl<'a, H: io::Handler<()>> io::Handler<()> for Handler<'a, H> {
  fn handle_event(&mut self, event: &io::Event, ctx: &mut io::Context) -> Option<()> {

    if ctx.socket().is_source_of(event) {
      match self.handle_socket_event(event, ctx) {
        Ok(result) => result,
        Err(err) => {
          println!("error while handling tls socket event: {:?}", err);
          Some( () )  //close the socket on error
        }
      }
    }
    else {
      //TODO handle closing tls socket here as well
      let (socket, child_ctx_factory) = ctx.as_socket_and_factory();
      let mut tls_socket = self.tls_context.wrap_socket(socket);
      let result = {
        let mut child_ctx = child_ctx_factory.into_context(&mut tls_socket);
        self.child_handler.handle_event(event, &mut child_ctx)
      };
      if result.is_some() {
        self.is_closing = true;
        tls_socket.flush();
        tls_socket.close();
        return None;
      }
      else {
        return None;
      }
    }
  }
}
