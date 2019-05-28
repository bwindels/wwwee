use super::ffi::*;
use super::{secret, x509, engine, Result};
use std::marker::PhantomData;
use std;
use std::ptr;

pub struct Context<'a> {
  ctx: Box<br_ssl_server_context>,
  lt: PhantomData<&'a u8>
}

impl<'a> Context<'a> {
  pub fn init_full_rsa(cert_chain: &'a [x509::Certificate<'a>], skey: Option<&'a secret::RsaKey>) -> Result<Context<'a>> {
    let mut ctx : Box<br_ssl_server_context> = Box::new(unsafe {
      std::mem::uninitialized()
    });
    let first_cert_ptr = cert_chain.get(0)
      .map_or(ptr::null(), |first_cert| first_cert.as_ptr());
    let skey_ptr = skey.map(|skey| skey.as_ptr()).unwrap_or(ptr::null());
    unsafe {
      br_ssl_server_init_full_rsa(
        &mut *ctx,
        first_cert_ptr,
        cert_chain.len(),
        skey_ptr);
    };
    Ok(Context {
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

  pub fn engine(&self) -> &engine::Context {
    &self.ctx.eng
  }

  pub fn engine_mut(&mut self) -> &mut engine::Context {
    &mut self.ctx.eng
  }

  pub fn set_single_rsa(&mut self, cert_chain: &'a [x509::Certificate<'a>], skey: &'a secret::RsaKey) {
    let first_cert_ptr = cert_chain.get(0)
      .map_or(ptr::null(), |first_cert| first_cert.as_ptr());
    unsafe {
      br_ssl_server_set_single_rsa(
        &mut *self.ctx,
        first_cert_ptr,
        cert_chain.len(),
        skey.as_ptr(),
        BR_KEYTYPE_KEYX | BR_KEYTYPE_SIGN,
        br_rsa_private_get_default(),
        br_rsa_pkcs1_sign_get_default());
    };
  }

  pub unsafe fn as_mut_ptr(&mut self) -> *mut br_ssl_server_context {
    &mut *self.ctx as *mut br_ssl_server_context
  }
}
