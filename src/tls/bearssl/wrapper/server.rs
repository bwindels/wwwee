use super::ffi::*;
use super::{secret, x509, engine, Result, Error};
use std::marker::PhantomData;
use std;

pub struct Context<'a> {
  ctx: Box<br_ssl_server_context>,
  lt: PhantomData<&'a u8>
}

impl<'a> Context<'a> {
  pub fn init_full_rsa(cert_chain: &'a [x509::Certificate<'a>], skey: &'a secret::RsaKey) -> Result<Context<'a>> {
    let mut ctx : Box<br_ssl_server_context> = Box::new(unsafe {
      std::mem::uninitialized()
    });
    let first_cert = cert_chain.get(0).ok_or(Error::BadLength)?;
    unsafe {
      br_ssl_server_init_full_rsa(
        &mut *ctx,
        first_cert.as_ptr(),
        cert_chain.len(),
        skey.as_ptr());
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

  pub unsafe fn as_mut_ptr(&mut self) -> *mut br_ssl_server_context {
    &mut *self.ctx as *mut br_ssl_server_context
  }
}
