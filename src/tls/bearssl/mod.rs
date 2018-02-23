mod ffi;

use std::marker::PhantomData;

enum Error {

}

pub struct X509Certificate {
  cert: ffi::br_x509_certificate
}

impl X509Certificate {
  pub fn new(certificate: &[u8]) -> X509Certificate {
    X509Certificate {
      cert: ffi::br_x509_certificate {
        data: certificate.as_ptr(),
        data_len: certificate.len()
      }
    }
  }
}

mod elliptic_curve {
  use super::ffi;

  pub enum Curve {

  }

  pub struct PrivateKey {
    key: ffi::br_ec_private_key
  }

  impl PrivateKey {
    pub fn new(private_key: &[u8], curve: Curve) -> PrivateKey {
      PrivateKey {
        key: ffi::br_ec_private_key {
          curve: curve as c_int,
          x: private_key.as_ptr(),
          xlen: private_key.len()
        }
      }
    }
  }
}

mod rsa {

}

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
    let buf = PageBuffer::new(ffi::BR_SSL_BUFSIZE_BIDI);
    const BI_DIRECTIONAL : c_int = 1;
    ffi::br_ssl_engine_set_buffer(&ctx.eng, buf.as_mut_ptr(), buf.len, BI_DIRECTIONAL);
    ffi::br_ssl_server_reset(&ctx.eng);
  }

  /// write source for incoming encrypted data
  pub fn record_input_channel(&mut self) -> Option<&WriteDst> {

  }

  /// reader for encrypted data to be sent to peer
  pub fn record_output_channel(&mut self) -> Option<&Readable> {

  }

  // writer for plaintext data to be decrypted and sent to peer
  pub fn decrypted_socket<'a>(&'a mut self) -> &DecryptedSocket<'a> {

  }
  
}

struct DecryptedSocket<'a> {
  ctx: &'a TLSContext
}

impl DecryptedSocket {
  pub fn can_read(&self) -> bool {

  }
  pub fn can_write(&self) -> bool {

  }
}

impl<'a> Read for DecryptedSocket<'a> {

}

impl<'a> Write for DecryptedSocket<'a> {

}


pub trait WriteDst {  //see Buffer
  fn read_from<R: io::Read>(&mut self, reader: &mut R) -> io::Result<usize>;
}

pub trait Readable {
  fn get_available() -> Option<&[u8]>;
}

//TLSHandler::handle_event: on socket readable event:
while let Some(dst) = ctx.record_input_channel() {
  dst.read_from(socket)?; //WouldBlock/EAGAIN here indicates nothing left to read
  //any internal TLS responses we can send straight away?
  if let Some(readable) = ctx.record_output_channel() {
    socket.write(readable.get_available())?;
  }

  let ds = ctx.decrypted_socket();
  if ds.can_read() {
    let event = event.with_borrowed_source(ds);
    handler.handle_event(&event)?;
  }
  //the handler probably wrote, so check again for responses
  if let Some(readable) = ctx.record_output_channel() {
    socket.write(readable.get_available())?;
  }
}

