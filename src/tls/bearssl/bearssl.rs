use super::ffi::*;
use std::marker::PhantomData;
use std;

pub type Result<T> = std::result::Result<T, Error>;

pub struct X509Certificate<'a> {
  cert: br_x509_certificate,
  lt: PhantomData<&'a u8>
}

impl<'a> X509Certificate<'a> {
  pub fn from_bytes(certificate: &mut [u8]) -> X509Certificate<'a> {
    X509Certificate {
      cert: br_x509_certificate {
        data: certificate.as_mut_ptr(),
        data_len: certificate.len()
      },
      lt: PhantomData
    }
  }

  pub unsafe fn as_ptr(&self) -> *const br_x509_certificate {
    std::mem::transmute(self)
  }
}

type RsaPrivateKey = br_rsa_private_key;
struct ServerContext<'a> {
  ctx: br_ssl_server_context,
  lt: PhantomData<&'a u8>
}

impl<'a> ServerContext<'a> {
  pub fn init_full_rsa(cert_chain: &'a [X509Certificate<'a>], skey: &'a RsaPrivateKey) -> Result<ServerContext<'a>> {
    let mut ctx : br_ssl_server_context = unsafe {
      std::mem::uninitialized()
    };
    let first_cert = cert_chain.get(0).ok_or(Error::BadLength)?;
    unsafe {
      br_ssl_server_init_full_rsa(
        &mut ctx,
        first_cert.as_ptr(),
        cert_chain.len(),
        skey as *const br_rsa_private_key);
    };
    Ok(ServerContext {
      ctx,
      lt: PhantomData
    })
  }

  pub fn reset(&mut self) -> Result<()> {
    unsafe {
      br_ssl_server_reset(self.as_mut_ptr())
    };
    self.engine().last_error()
  }

  pub fn engine(&self) -> &EngineContext {
    &self.ctx.eng
  }

  pub fn engine_mut(&mut self) -> &mut EngineContext {
    &mut self.ctx.eng
  }

  pub unsafe fn as_mut_ptr(&mut self) -> *mut br_ssl_server_context {
    std::mem::transmute(self)
  }
}

type EngineContext = br_ssl_engine_context;

impl EngineContext {
  pub fn recvrec_buf<'a>(&'a mut self) -> Option<&'a mut [u8]> {
    let mut size = 0usize;
    let ptr = unsafe {
      br_ssl_engine_recvrec_buf(
        self as *const EngineContext,
        &mut size as *mut usize)
    };
    ptr_to_slice(ptr, size)
  }

  pub fn recvrec_ack(&mut self, len: usize) -> Result<()> {
    unsafe {
      br_ssl_engine_recvrec_ack(self as *mut EngineContext, len)
    }
    self.last_error()
  }

  pub fn sendrec_buf<'a>(&'a mut self) -> Option<&'a mut [u8]> {
    let mut size = 0usize;
    let ptr = unsafe {
      br_ssl_engine_sendrec_buf(
        self as *const EngineContext,
        &mut size as *mut usize)
    };
    ptr_to_slice(ptr, size)
  }

  pub fn sendrec_ack(&mut self, len: usize) -> Result<()> {
    unsafe {
      br_ssl_engine_sendrec_ack(self as *mut EngineContext, len)
    }
    self.last_error()
  }

  pub fn recvapp_buf<'a>(&'a mut self) -> Option<&'a mut [u8]> {
    let mut size = 0usize;
    let ptr = unsafe {
      br_ssl_engine_recvapp_buf(
        self as *const EngineContext,
        &mut size as *mut usize)
    };
    ptr_to_slice(ptr, size)
  }

  pub fn recvapp_ack(&mut self, len: usize) -> Result<()> {
    unsafe {
      br_ssl_engine_recvapp_ack(self as *mut EngineContext, len)
    }
    self.last_error()
  }

  pub fn sendapp_buf<'a>(&'a mut self) -> Option<&'a mut [u8]> {
    let mut size = 0usize;
    let ptr = unsafe {
      br_ssl_engine_sendapp_buf(
        self as *const EngineContext,
        &mut size as *mut usize)
    };
    ptr_to_slice(ptr, size)
  }

  pub fn sendapp_ack(&mut self, len: usize) -> Result<()> {
    unsafe {
      br_ssl_engine_sendapp_ack(self as *mut EngineContext, len)
    }
    self.last_error()
  }

  pub fn last_error(&self) -> Result<()> {
    if self.err == BR_ERR_OK as i32 {
      Ok(())
    }
    else {
      let err = unsafe { std::mem::transmute(self.err as i16) };
      Err(err)
    }
  }
}

fn ptr_to_slice<'a>(ptr: *mut std::os::raw::c_uchar, len: usize) -> Option<&'a mut [u8]> {
  not_null_mut(ptr).map(|ptr| {
    unsafe {
      std::slice::from_raw_parts_mut(ptr as *mut u8, len)
    }
  })
}

fn not_null_mut<T>(ptr: *mut T) -> Option<*mut T> {
  if ptr.is_null() {
    None
  }
  else {
    Some(ptr)
  }
}

pub enum Error {
  BadParam = BR_ERR_BAD_PARAM as isize,
  BadState = BR_ERR_BAD_STATE as isize,
  UnsupportedVersion = BR_ERR_UNSUPPORTED_VERSION as isize,
  BadVersion = BR_ERR_BAD_VERSION as isize,
  BadLength = BR_ERR_BAD_LENGTH as isize,
  TooLarge = BR_ERR_TOO_LARGE as isize,
  BadMac = BR_ERR_BAD_MAC as isize,
  NoRandom = BR_ERR_NO_RANDOM as isize,
  UnknownType = BR_ERR_UNKNOWN_TYPE as isize,
  Unexpected = BR_ERR_UNEXPECTED as isize,
  BadCcs = BR_ERR_BAD_CCS as isize,
  BadAlert = BR_ERR_BAD_ALERT as isize,
  BadHandshake = BR_ERR_BAD_HANDSHAKE as isize,
  OversizedId = BR_ERR_OVERSIZED_ID as isize,
  BadCipherSuite = BR_ERR_BAD_CIPHER_SUITE as isize,
  BadCompression = BR_ERR_BAD_COMPRESSION as isize,
  BadFraglen = BR_ERR_BAD_FRAGLEN as isize,
  BadSecreneg = BR_ERR_BAD_SECRENEG as isize,
  ExtraExtension = BR_ERR_EXTRA_EXTENSION as isize,
  BadSni = BR_ERR_BAD_SNI as isize,
  BadHelloDone = BR_ERR_BAD_HELLO_DONE as isize,
  LimitExceeded = BR_ERR_LIMIT_EXCEEDED as isize,
  BadFinished = BR_ERR_BAD_FINISHED as isize,
  ResumeMismatch = BR_ERR_RESUME_MISMATCH as isize,
  InvalidAlgorithm = BR_ERR_INVALID_ALGORITHM as isize,
  BadSignature = BR_ERR_BAD_SIGNATURE as isize,
  WrongKeyUsage = BR_ERR_WRONG_KEY_USAGE as isize,
  NoClientAuth = BR_ERR_NO_CLIENT_AUTH as isize,
  Io = BR_ERR_IO as isize,
  RecvFatalAlert = BR_ERR_RECV_FATAL_ALERT as isize,
  SendFatalAlert = BR_ERR_SEND_FATAL_ALERT as isize,
}


/*
type X509DecoderContext = br_x509_decoder_context;

unsafe extern "C"
fn write_dn(ctx: *mut c_void, buf: *const c_void, len: usize) {

}

impl X509DecoderContext {
  pub fn init(dn_writer: &mut std::io::Write) -> X509DecoderContext {
    let mut dc : br_x509_decoder_context = unsafe {
      std::mem::uninitialized()
    };
    let append_dn_ctx : *mut c_void = unsafe {
      std::mem::transmute(dn_writer)
    };
    unsafe {
      br_x509_decoder_init(
        &mut dc as *mut X509DecoderContext,
        Some(write_dn),
        append_dn_ctx)
    };
    dc
  }

}*/
