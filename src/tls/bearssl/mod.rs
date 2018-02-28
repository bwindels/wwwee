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
  pub fn decrypted_socket<'a>(&'a mut self) -> &DecryptedSocket<'a> {
    DecryptedSocket {ctx: &mut self}
  }
}

struct DecryptedSocket<'a> {
  ctx: &mut 'a TLSContext
}

impl DecryptedSocket {
  pub fn can_read(&self) -> bool {
    ffi::br_ssl_engine_recvapp_buf(&ctx.ctx).is_some()
  }

  pub fn can_write(&self) -> bool {
    ffi::br_ssl_engine_sendapp_buf(&ctx.ctx).is_some()
  }
}

impl<'a> Read for DecryptedSocket<'a> {
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

impl<'a> ReadSizeHint for DecryptedSocket<'a> {
  fn read_size_hint(&self) -> Option<usize> {
    let mut size = 0usize;
    ffi::br_ssl_engine_recvapp_buf(&ctx.ctx, &mut size).map(|_| size)
  }
}


impl<'a> Write for DecryptedSocket<'a> {
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

pub struct ReceiveRecordBuffer<'a> {
  ctx: &mut 'a ffi::br_ssl_server_context,
  buffer: &mut 'a [u8]
}

impl<'a> ReadDst for ReceiveRecordBuffer<'a> {
  fn read_from<R: io::Read>(self, reader: &mut R) -> io::Result<usize> {
    reader.read(buffer).map(|bytes_read| {
      ffi::br_ssl_engine_sendrec_ack(self.ctx, bytes_read);
      bytes_read
    })
  }
}

pub struct SendRecordBuffer<'a> {
  ctx: &mut 'a ffi::br_ssl_server_context,
  buffer: &mut 'a [u8]
}

impl<'a> WriteSrc for SendRecordBuffer<'a> {
  fn write_to<R: io::Write>(self, writer: &mut W) -> io::Result<usize> {
    writer.write(slice).map(|bytes_written| {
      ffi::br_ssl_engine_sendrec_ack(self.ctx, bytes_written);
      bytes_written
    });
  }
}

/// trait similar to Write but assumes it already has an internal buffer
/// that can be written (read into) from R without copying.
/// Using std::io::Read and std::io::Write would require an intermediate buffer.
pub trait ReadDst {
  fn read_from<R: io::Read>(&mut self, reader: &mut R) -> io::Result<usize>;
  fn read_from_with_hint<R: io::Read + ReadSizeHint>(&mut self, reader: &mut R) -> io::Result<usize> {
    self.read_from(reader)
  }
}

pub trait ReadSizeHint {
  fn read_size_hint(&self) -> Option<usize>;
}

/// trait similar to Read but assumes it already has an internal buffer that
/// can be writen to W without copying
/// Using std::io::Read and std::io::Write would require an intermediate buffer.
pub trait WriteSrc {
  fn write_to<R: io::Write>(&mut self, writer: &mut W) -> io::Result<usize>;
}

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
