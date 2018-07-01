use std;
use std::io::Write;
use io;
use super::context::Context;

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
        //tls termination finished,
        //terminate the connection
        return Ok( Some( () ) );
      }
      else {
        //wait for more events
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
        let mut child_ctx = child_ctx_factory.into_context(&mut tls_socket);
        let result = self.child_handler.handle_event(&child_event, &mut child_ctx);
        Ok(result)
      }
      else {
        println!("waiting for more events before forwarding to child handler");
        Ok(None) //need more events
      }
    }
  }

  fn start_tls_session_termination(&mut self, ctx: &mut io::Context) -> std::io::Result<()> {
    println!("closing tls socket");
    self.is_closing = true;
    let (socket, _) = ctx.as_socket_and_factory();
    let mut tls_socket = self.tls_context.wrap_socket(socket);
    tls_socket.flush()?;
    tls_socket.close()?;
    Ok( () )
  }
}

impl<'a, H: io::Handler<()>> io::Handler<()> for Handler<'a, H> {
  fn handle_event(&mut self, event: &io::Event, ctx: &mut io::Context) -> Option<()> {

    //if an event on the socket, handle it seperatly
    //because tls handshake needs to be handled here
    let mut result = if ctx.socket().is_source_of(event) {
      self.handle_socket_event(event, ctx)
    }
    // as app data shouldn't be sent/received anymore
    // after starting to close the tls session,
    // don't deliver non-socket events
    // (e.g. file data available from reader) anymore
    // if this is the case.
    else if !self.is_closing {
      let (socket, child_ctx_factory) = ctx.as_socket_and_factory();
      let mut tls_socket = self.tls_context.wrap_socket(socket);
      let mut child_ctx = child_ctx_factory.into_context(&mut tls_socket);
      let result = self.child_handler.handle_event(event, &mut child_ctx);
      Ok( result )
    }
    // keep connection open and wait for more events
    else {
      Ok ( None )
    };
    // if the child handler finished, terminate tls session
    if let Ok(Some(_)) = result {
      result = self
        .start_tls_session_termination(ctx)
        .map(|_| None); //None: wait for more events to finish tls termination
    }

    match result {
      Ok(result) => result,
      Err(_) => Some( () )  //on error, close socket
    }
  }
}
