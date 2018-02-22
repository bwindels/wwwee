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

mod eliptic_curve {
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

pub struct TLSContext<'a> {
  br_server_ctx: ffi::br_ssl_server_context,
}

impl<'a> TLSContext<'a> {
  pub fn from_certificate(
    certificate_chain: &[X509Certificate],
    key: &PrivateKey
  ) -> Result<TLSContext<'a>, Error> {
    let ctx : ffi::br_ssl_server_context = unsafe { std::mem::zeroed() };
    ffi::br_ssl_server_init_full_ec(
      &ctx as *mut ffi::br_ssl_server_context,
      certificate_chain,
      certificate_chain.len(),
      ffi::BR_KEYTYPE_RSA,  //lets encrypt will only support EC signatures in Q3 2018
      key
    );
    //br_ssl_engine_set_buffer
    //br_ssl_server_reset
  }

  pub fn application_sink(&mut self) -> &AppSink {

  }

  pub fn transportation_sink(&mut self) -> &TransportSink {

  }
}

pub struct AppSink {
  ctx: &TLSContext
}

impl Write for AppSink {

}

impl Readable for AppSink {

}

pub struct TransportSink {
  ctx: &TLSContext
}

impl ReadFrom for TransportSink {

}

impl Readable for TransportSink {
}

pub trait Readable {
  fn get_available<'a>(&'a self) -> Option<&'a [u8]>;
}

pub trait ReadFrom {  //see Buffer
  fn read_from<R: io::Read>(&mut self, reader: &mut R) -> io::Result<usize>;
}


/*
Read + ReadFrom means copy, which we only want from os socket
in between handlers, we can avoid copy by using ...
we want to pass the socket abstraction in the event, handlers should not own it anymore
much like bytes_available event we had in the beginning

but in event put newly received bytes (like we have to do with tls because buffers are reused)
or all bytes like the append buffer we have in http handler?
doing both over same trait would be confusing semantics
*/

trait SocketAbstraction : Readable + Write {

}

//OR

enum Event<W: io::Write> {
  IncomingData(&'a [u8], W),  //but how would that work with files, etc ???
}



/*
once decrypted, we want a buffer that grows in page increments to contain all headers
we will probably have to copy the data from the recvapp buffer to the growable buffer,
or could bearssl directly append into this structure?
*/

/*
so the IO pattern could be, on receiving socket data:
ctx.transportation_sink().write(data)
if let Some(buffer) = ctx.transportation_sink().get_available() {
  socket.write(buffer)
}
if let Some(buffer) = ctx.application_sink().get_available() {
  handler(buffer as reader + application_sink().write) //pass buffer as Read and app sink as write
}
*/
