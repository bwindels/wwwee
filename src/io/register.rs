use std;
use mio;
use std::ops::{Deref, DerefMut};
use super::{Token, AsyncToken, Context};

pub trait Register {
  fn register(&mut self, selector: &mio::Poll, token: Token) -> std::io::Result<()>;
  fn deregister(&mut self, selector: &mio::Poll) -> std::io::Result<()>;
}

pub struct Registered<R> {
  registerable: R,
  token: AsyncToken
}

impl<R: Register> Registered<R> {
  pub fn register(mut registerable: R, token: Token, selector: &mio::Poll) -> std::io::Result<Registered<R>> {
    registerable.register(selector, token)?;
    Ok(Registered {registerable, token: token.async_token()})
  }

  pub fn into_deregistered(mut self, ctx: &Context) -> std::io::Result<R> {
    ctx.deregister(&mut self.registerable)?;
    Ok(self.registerable)
  }

  pub fn token(&self) -> AsyncToken {
    self.token
  }
}

impl<R> Deref for Registered<R> {
  type Target = R;

  fn deref(&self) -> &Self::Target {
    &self.registerable
  }
} 

impl<R> DerefMut for Registered<R> {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.registerable
  }
}
