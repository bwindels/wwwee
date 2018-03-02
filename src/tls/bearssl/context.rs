use ::buffer::PageBuffer;
use super::wrapper::*;
use super::{ReceiveRecordBuffer, SendRecordBuffer};

pub struct TLSContext<'a> {
  buffer: PageBuffer,
  server_context: server::Context<'a>,
}

impl<'a> TLSContext<'a> {
  pub fn from_certificate(
    certificate_chain: &'a [x509::Certificate<'a>],
    key: &'a secret::Key<'a>)
  -> Result<TLSContext<'a>>
  {
    let mut server_context = match key {
      &secret::Key::Rsa(rsa_key) => {
        server::Context::init_full_rsa(certificate_chain, rsa_key)?
      },
      _ => unimplemented!("only RSA keys are implemented for now")
    };

    // TODO: check if we can make buffer smaller/non-bidi if we always empty it straight away
    // probably yes, we plan to drain the recvapp buffer as soon as we can
    let mut buffer = PageBuffer::new(ffi::BR_SSL_BUFSIZE_BIDI as usize);
    server_context.engine_mut().set_buffer(buffer.as_mut_slice(), true);
    server_context.reset()?;
    server_context.engine().last_error().map(|_| {
      TLSContext {
        buffer,
        server_context
      }
    })
  }

  /// write source for incoming encrypted data
  pub fn receive_record_channel(&'a mut self) -> Option<ReceiveRecordBuffer<'a>> {
    let engine = self.server_context.engine_mut();
    let buffer_available = engine.recvrec_buf().is_some();
    if buffer_available {
      Some(ReceiveRecordBuffer::new(engine))
    }
    else {
      None
    }
  }

  /// reader for encrypted data to be sent to peer
  pub fn send_record_channel(&'a mut self) -> Option<SendRecordBuffer<'a>> {
    let engine = self.server_context.engine_mut();
    let buffer_available = engine.sendrec_buf().is_some();
    if buffer_available {
      Some(SendRecordBuffer::new(engine))
    }
    else {
      None
    }
  }
/*
  // reader/writer for plaintext data
  pub fn wrap_socket(&'a mut self, socket: &io::Socket) -> &WrappedSocket<'a> {
    WrappedSocket {ctx: &mut self, socket}
  }
*/
}
