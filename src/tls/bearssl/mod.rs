mod ffi;
mod x509;
mod context;
mod error;
mod skey;
mod socket;
mod record_channels;

//TLSHandler::handle_event: on socket readable event:

loop {
  self.is_writable = self.is_writable || event.is_writable();
  if event.token() == socket.token() {
    if event.is_readable() {
      if let Some(receive_channel) = tls_context.receive_record_channel() {
        receive_channel.read_from(socket)?; //WouldBlock/EAGAIN here indicates nothing left to read
        //any internal TLS responses we can send straight away?
        if let Some(send_channel) = tls_context.send_record_channel() {
          send_channel.write_to(socket)?;
        }
      }
      else {
        // Aaargh, what to do here? We've got data but bearssl is not ready to receive it!
      }
    }
    if self.is_writable {
      //try to write anything we've got (e.g . last records of end of response)
      while let Some(send_channel) = tls_context.send_record_channel() {
        send_channel.write_to(socket)?;
      }
    }
  }
  let s = tls_context.wrap_socket(socket);
  if s.can_read() {
    let ctx = ctx.with_wrapped_socket(s);
    handler.handle_event(event, &ctx)?;
  }
  // need to flush here?
}
