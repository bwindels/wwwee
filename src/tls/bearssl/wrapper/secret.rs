use super::ffi::*;
use std;
use std::os::raw::c_void;
use super::x509;

pub type DecoderContext = br_skey_decoder_context;

impl DecoderContext {

  pub fn new() -> DecoderContext {
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

  pub fn get_key(&self)
    -> std::result::Result<Key, x509::Error>
  {
    self.last_error().and_then(|_| {
      match self.key_type as u32 {
        BR_KEYTYPE_RSA => {
          let rsa_key = RsaKey::from(& unsafe { self.key.rsa }); //access to union field
          Ok(Key::Rsa(rsa_key))
        },
        BR_KEYTYPE_EC => {
          let ec_key = EcKey::from(& unsafe { self.key.ec }); //access to union field
          Ok(Key::Ec(ec_key))
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

pub enum Key {
  Rsa(RsaKey),
  Ec(EcKey)
}

pub struct RsaKey {
  skey: br_rsa_private_key,
  key_data: Vec<u8>
}

impl RsaKey {

  pub fn from(skey: &br_rsa_private_key) -> RsaKey {
    let mut skey = skey.clone();
    let key_data = {
      let mut key_parts : [(&mut *mut u8, usize); 5] = [
        (&mut skey.p,  skey.plen),
        (&mut skey.q,  skey.qlen),
        (&mut skey.dp, skey.dplen),
        (&mut skey.dq, skey.dqlen),
        (&mut skey.iq, skey.iqlen),
      ];

      let total_len = key_parts.iter().fold(0, |acc, part| acc + part.1);
      let mut key_data : Vec<u8> = Vec::with_capacity(total_len);

      key_parts.iter_mut().fold(0, |offset, part| {
        let len = part.1;
        let slice : &[u8] = unsafe { std::slice::from_raw_parts(*part.0, len) };
        key_data.extend_from_slice(slice);
        *part.0 = unsafe { key_data.as_mut_ptr().offset(offset as isize) };
        offset + len
      });
      key_data
    };

    RsaKey { key_data, skey }
  }

  pub fn as_ptr(&self) -> *const br_rsa_private_key {
    &self.skey as *const br_rsa_private_key
  }
}

pub struct EcKey {
  skey: br_ec_private_key,
  key_data: Vec<u8>
}

impl<'a> EcKey {

  pub fn from(skey: &br_ec_private_key) -> EcKey {
    let mut skey = skey.clone();
    let mut key_data = Vec::with_capacity(skey.xlen);
    let x = unsafe { std::slice::from_raw_parts(skey.x, skey.xlen) };
    key_data.extend_from_slice(x);
    skey.x = key_data.as_mut_ptr();
    EcKey { skey, key_data }
  }

  pub fn as_ptr(&self) -> *const br_ec_private_key {
    &self.skey as *const br_ec_private_key
  }

}
