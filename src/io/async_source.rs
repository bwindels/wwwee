use std;
use mio;
use std::ops::{Deref, DerefMut};
use super::{Token, EventKind, Event, AsyncToken, Context};

pub trait AsyncSource {
  fn register(&mut self, selector: &mio::Poll, token: Token) -> std::io::Result<()>;
  fn deregister(&mut self, selector: &mio::Poll) -> std::io::Result<()>;
  fn is_registered_event_kind(&self, kind: EventKind) -> bool;
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

  pub fn is_source_of(&self, event: &Event) -> bool {
    event.token() == self.token && self.source.is_registered_event_kind(event.kind())
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
