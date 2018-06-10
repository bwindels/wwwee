use io;

pub struct Handler<H> {
  tls_context: Context,
  child_handler: H,
  is_writable: bool
}

/*
here we need to
- handle gracefully when the socket isn't writable (wait for next event to send)
- check upon receiving records whether there is something to send out again (handshake!)
- check if there is plaintext available, if so, wrap the socket and forward the event to child_handler 
*/

impl<T, H: io::Handler<T>> io::Handler<T> for Handler<H> {
  fn handle_event(&mut self, event: &io::Event, ctx: &io::Context) -> Option<T> {

    loop {
      self.is_writable = self.is_writable || event.is_writable();
      if event.token() == socket.token() {
        if event.is_readable() {
          if let Some(receive_channel) = self.tls_context.receive_record_channel() {
            receive_channel.read_from(socket)?; //WouldBlock/EAGAIN here indicates nothing left to read
            //any internal TLS responses we can send straight away?
            if let Some(send_channel) = self.tls_context.send_record_channel() {
              send_channel.write_to(socket)?;
            }
          }
          else {
            // Aaargh, what to do here? We've got data but bearssl is not ready to receive it!
          }
        }
        if self.is_writable {
          //try to write anything we've got (e.g . last records of end of response)
          while let Some(send_channel) = self.tls_context.send_record_channel() {
            send_channel.write_to(socket)?;
          }
        }
      }
      let s = self.tls_context.wrap_socket(socket);
      if s.can_read() {
        let ctx = ctx.with_wrapped_socket(s);
        self.child_handler.handle_event(event, &ctx)?;
      }
      // need to flush here?
    }

  }
}
