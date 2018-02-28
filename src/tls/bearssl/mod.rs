mod ffi;

//TLSHandler::handle_event: on socket readable event:

loop {
  if let Some(receive_channel) = ctx.receive_record_channel() {
    receive_channel.read_from(socket)?; //WouldBlock/EAGAIN here indicates nothing left to read
  }
  //any internal TLS responses we can send straight away?
  if let Some(send_channel) = ctx.send_record_channel() {
    send_channel.write_to(socket)?;
  }
  let ds = ctx.decrypted_socket();
  if ds.can_read() {
    let event = event.with_socket_wrapper(ds);
    handler.handle_event(&event)?;
  }
  //the handler probably wrote, so check again for responses
  if let Some(src) = ctx.send_record_channel() {
    send_channel.write_to(socket)?;
  }
}
