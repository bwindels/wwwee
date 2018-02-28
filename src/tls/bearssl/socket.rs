
struct DecryptedSocket<'a, W> {
  ctx: &mut 'a TLSContext
  rec_writer: &mut 'a W 
}

impl<'a> DecryptedSocket<'a, W> {
  pub fn can_read(&self) -> bool {
    ffi::br_ssl_engine_recvapp_buf(&ctx.ctx).is_some()
  }

  pub fn can_write(&self) -> bool {
    ffi::br_ssl_engine_sendapp_buf(&ctx.ctx).is_some()
  }
}

impl<'a, W: io::Write> io::Read for DecryptedSocket<'a, W> {
  fn read(&mut self, dst_buffer: &mut [u8]) -> io::Result<usize> {
    let mut size = 0usize;
    ffi::br_ssl_engine_recvapp_buf(&ctx.ctx.eng, &mut size).map(|ptr| {
      let src_buffer = slice::from_raw_parts(ptr, size);
      let len = cmp::min(src_buffer.len(), dst_buffer.len());
      ptr::copy_non_overlapping(src_buffer, dst_buffer, len);
      ffi::br_ssl_engine_recvapp_ack(&ctx.ctx.eng, len);
      len
    }).ok_or(/*invalid state error*/)
  }
}

impl<'a, W> ReadSizeHint for DecryptedSocket<'a, W> {
  fn read_size_hint(&self) -> Option<usize> {
    let mut size = 0usize;
    ffi::br_ssl_engine_recvapp_buf(&ctx.ctx, &mut size).map(|_| size)
  }
}


impl<'a, W: io::Write> io::Write for DecryptedSocket<'a, W> {
  // TODO: this will need to flush to socket when sendrec is available
  fn write(&mut self, src_buffer: &[u8]) -> io::Result<usize> {
    let mut size = 0usize;
    ffi::br_ssl_engine_sendapp_buf(&ctx.ctx.eng, &mut size).map(|ptr| {
      let dst_buffer = slice::from_raw_parts(ptr, size);
      let len = cmp::min(src_buffer.len(), dst_buffer.len());
      ptr::copy_non_overlapping(src_buffer, dst_buffer, len);
      ffi::br_ssl_engine_sendapp_ack(&ctx.ctx.eng, len);
      len
    }).ok_or(/*invalid state error*/)
  }
}