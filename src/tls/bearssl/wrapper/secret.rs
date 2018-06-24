use super::ffi::*;
use std::marker::PhantomData;
use std;
use std::os::raw::c_void;
use super::x509;

pub struct DecoderContext<'a> {
  ctx: br_skey_decoder_context,
  phantom_data: PhantomData<&'a u8>
}

impl<'a> DecoderContext<'a> {
  /*pub fn init() -> DecoderContext {
    let mut ctx : br_skey_decoder_context = unsafe {
      std::mem::uninitialized()
    };
    unsafe {
      br_skey_decoder_init(&mut ctx as *mut br_skey_decoder_context)
    };
    ctx
  }*/

  pub fn from_bytes(skey_der_bytes: &'a [u8]) -> DecoderContext<'a> {
    let mut ctx : br_skey_decoder_context = unsafe {
      std::mem::uninitialized()
    };
    unsafe {
      let ctx_ptr = &mut ctx as *mut br_skey_decoder_context;
      br_skey_decoder_init(ctx_ptr);
      br_skey_decoder_push(
        ctx_ptr,
        skey_der_bytes.as_ptr() as *const c_void,
        skey_der_bytes.len());
    };
    DecoderContext { ctx, phantom_data: PhantomData }
  }
  /*
  pub fn push(&mut self, buf: &[u8]) {
    unsafe {
      br_skey_decoder_push(
        self as *mut br_skey_decoder_context,
        buf.as_ptr() as *const c_void,
        buf.len())
    }
  }
  */

  pub fn get_key<'s>(&'s self)
    -> std::result::Result<Key<'a>, x509::Error>
    where 'a: 's 
  {
    self.last_error().and_then(|_| {
      match self.ctx.key_type as u32 {
        BR_KEYTYPE_RSA => {
          let rsa_key = RsaKey {
            skey: unsafe { self.ctx.key.rsa }, //access to union field
            phantom_data: PhantomData
          };
          Ok(Key::Rsa(rsa_key))
        },
        BR_KEYTYPE_EC => {
          let ec_key = EcKey {
            skey: unsafe { self.ctx.key.ec }, //access to union field
            phantom_data: PhantomData
          };
          Ok(Key::Ec(ec_key))
        },
        _ => Err(x509::Error::WrongKeyType)
      }
    })
  }

  fn last_error(&self) -> std::result::Result<(), x509::Error> {
    if self.ctx.err != 0 as i32 {
      let err = unsafe { std::mem::transmute(self.ctx.err as i8) };
      Err(err)
    }
    else if self.ctx.key_type == 0 {
      Err(x509::Error::Truncated)
    }
    else {
      Ok( () )
    }
  }
}

pub enum Key<'a> {
  Rsa(RsaKey<'a>),
  Ec(EcKey<'a>)
}

pub struct RsaKey<'a> {
  skey: br_rsa_private_key,
  phantom_data: PhantomData<&'a u8>
}

impl<'a> RsaKey<'a> {
  pub fn as_ptr(&self) -> *const br_rsa_private_key {
    &self.skey as *const br_rsa_private_key
  }
}

pub struct EcKey<'a> {
  skey: br_ec_private_key,
  phantom_data: PhantomData<&'a u8>
}

impl<'a> EcKey<'a> {
  pub fn as_ptr(&self) -> *const br_ec_private_key {
    &self.skey as *const br_ec_private_key
  }
}
