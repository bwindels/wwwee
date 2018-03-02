use super::ffi::*;
use std::marker::PhantomData;
use std;
use std::os::raw::c_void;
use super::x509;

pub type DecoderContext = br_skey_decoder_context;

impl DecoderContext {
  pub fn init() -> DecoderContext {
    let mut ctx : br_skey_decoder_context = unsafe {
      std::mem::uninitialized()
    };
    unsafe {
      br_skey_decoder_init(&mut ctx as *mut br_skey_decoder_context)
    };
    ctx
  }

  pub fn push(&mut self, buf: &[u8]) {
    unsafe {
      br_skey_decoder_push(
        self as *mut br_skey_decoder_context,
        buf.as_ptr() as *const c_void,
        buf.len())
    }
  }

  pub fn get_key<'a>(&'a self) -> std::result::Result<Key<'a>, x509::Error> {
    self.last_error().and_then(|_| {
      match self.key_type as u32 {
        BR_KEYTYPE_RSA => {
          let rsa_key_ref = unsafe{
            std::mem::transmute(&self.key.rsa)
          };
          Ok(Key::Rsa(rsa_key_ref))
        },
        BR_KEYTYPE_EC => {
          let ec_key_ref = unsafe {
            std::mem::transmute(&self.key.ec)
          };
          Ok(Key::Ec(ec_key_ref))
        },
        _ => Err(x509::Error::WrongKeyType)
      }
    })
  }

  fn last_error(&self) -> std::result::Result<(), x509::Error> {
    if self.err != 0 as i32 {
      let err = unsafe { std::mem::transmute(self.err as i8) };
      Err(err)
    }
    else if self.key_type == 0 {
      Err(x509::Error::Truncated)
    }
    else {
      Ok( () )
    }
  }
}

pub enum Key<'a> {
  Rsa(&'a RsaKey<'a>),
  Ec(&'a EcKey<'a>)
}

pub struct RsaKey<'a> {
  skey: br_rsa_private_key,
  lt: PhantomData<&'a u8>
}

impl<'a> RsaKey<'a> {
  pub fn as_ptr(&self) -> *const br_rsa_private_key {
    &self.skey as *const br_rsa_private_key
  }
}

pub struct EcKey<'a> {
  skey: br_ec_private_key,
  lt: PhantomData<&'a u8>
}

impl<'a> EcKey<'a> {
  pub fn as_ptr(&self) -> *const br_ec_private_key {
    &self.skey as *const br_ec_private_key
  }
}
