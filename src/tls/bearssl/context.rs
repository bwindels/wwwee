use io;
use ::buffer::PageBuffer;
use super::wrapper::*;
use super::socket::SocketWrapper;
use super::{ReceiveRecordChannel, SendRecordChannel};

/*
there are 4 buffers in the context:
2 app buffers, which deal with plaintext
2 rec(ord) buffers, which deal with encrypted data
both the app and rec buffer pairs have send and receive buffers
so:
  recvrec buffer is for encrypted data read from the socket 
  sendrec buffer is for encrypted data to write to the socket 
  recvapp buffer is for decrypted data to process
  sendapp buffer is for data to be encrypted and sent over the socket 
*/

pub struct Context<'a> {
  buffer: PageBuffer,
  server_context: server::Context<'a>,
}

impl<'a> Context<'a> {
  pub fn from_certificate(
    certificate_chain: &'a [x509::Certificate<'a>],
    key: &'a secret::Key<'a>)
  -> Result<Context<'a>>
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
      Context {
        buffer,
        server_context
      }
    })
  }

  /// write source for incoming encrypted data
  pub fn receive_record_channel(&'a mut self) -> Option<ReceiveRecordChannel<'a>> {
    let engine = self.server_context.engine_mut();
    let buffer_available = engine.recvrec_buf().is_some();
    if buffer_available {
      Some(ReceiveRecordChannel::new(engine))
    }
    else {
      None
    }
  }

  /// reader for encrypted data to be sent to peer
  pub fn send_record_channel(&'a mut self) -> Option<SendRecordChannel<'a>> {
    let engine = self.server_context.engine_mut();
    let buffer_available = engine.sendrec_buf().is_some();
    if buffer_available {
      Some(SendRecordChannel::new(engine))
    }
    else {
      None
    }
  }

  pub fn wrap_socket(&'a mut self, socket: &'a mut io::Socket) -> SocketWrapper<'a> {
    SocketWrapper::new(&mut self.server_context.engine_mut(), socket)
  }
}
