
pub struct TLSContext<'a> {
  br_server_ctx: ffi::br_ssl_server_context,
}

impl<'a> TLSContext<'a> {
  pub fn from_certificate(
    certificate_chain: &[X509Certificate],
    key: &PrivateKey
  ) -> Result<TLSContext<'a>, Error> {
    let ctx : ffi::br_ssl_server_context = unsafe { std::mem::zeroed() };
    ffi::br_ssl_server_init_full_rsa(
      &ctx as *mut ffi::br_ssl_server_context,
      certificate_chain,
      certificate_chain.len(),
      ffi::BR_KEYTYPE_RSA,
      key
    );
    // TODO: check if we can make buffer smaller/non-bidi if we always empty it straight away
    let buf = PageBuffer::new(ffi::BR_SSL_BUFSIZE_BIDI);
    const BI_DIRECTIONAL : c_int = 1;
    ffi::br_ssl_engine_set_buffer(&ctx.eng, buf.as_mut_ptr(), buf.len, BI_DIRECTIONAL);
    ffi::br_ssl_server_reset(&ctx.eng);
  }

  /// write source for incoming encrypted data
  pub fn receive_record_channel(&mut self) -> Option<ReceiveRecordBuffer> {
    let mut size = 0usize;
    ffi::br_ssl_engine_recvrec_buf(self.ctx, &size)
      .map(|ptr| {
        let buffer = std::slice::from_raw_parts(ptr, size);
        ReceiveRecordBuffer {ctx: &mut self.ctx, buffer}
      });
  }

  /// reader for encrypted data to be sent to peer
  pub fn send_record_channel(&mut self) -> Option<&SendRecordBuffer> {
    let mut size = 0usize;
    ffi::br_ssl_engine_sendrec_buf(self.ctx, &size)
      .map(|ptr| {
        let buffer = std::slice::from_raw_parts(ptr, size);
        SendRecordBuffer {ctx: &mut self.ctx, buffer}
      });
  }

  // reader/writer for plaintext data
  pub fn decrypted_socket<'a, W: io::Write>(&'a mut self, rec_writer: &W) -> &DecryptedSocket<'a> {
    DecryptedSocket {ctx: &mut self, rec_writer: rec_writer}
  }
}