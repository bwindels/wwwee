use std;
use mio;
use std::ops::{Deref, DerefMut};
use super::{Token, AsyncToken, Context};

pub trait AsyncSource {
  fn register(&mut self, selector: &mio::Poll, token: Token) -> std::io::Result<()>;
  fn deregister(&mut self, selector: &mio::Poll) -> std::io::Result<()>;
}

pub struct Registered<R> {
  source: R,
  token: AsyncToken
}

impl<R: AsyncSource> Registered<R> {
  pub fn register(mut source: R, token: Token, selector: &mio::Poll) -> std::io::Result<Registered<R>> {
    source.register(selector, token)?;
    Ok(Registered {source, token: token.async_token()})
  }

  pub fn into_deregistered(mut self, ctx: &Context) -> std::io::Result<R> {
    ctx.deregister(&mut self.source)?;
    Ok(self.source)
  }

  pub fn token(&self) -> AsyncToken {
    self.token
  }
}

impl<R> Deref for Registered<R> {
  type Target = R;

  fn deref(&self) -> &Self::Target {
    &self.source
  }
}

impl<R> DerefMut for Registered<R> {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.source
  }
}
