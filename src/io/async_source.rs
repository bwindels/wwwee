use std;
use mio;
use std::ops::{Deref, DerefMut};
use super::{Token, Event, AsyncToken, Context, ReadSizeHint};
use std::io::{Write, Read};

pub trait EventSource {
  fn token(&self) -> AsyncToken;
  
  fn is_source_of(&self, event: &Event) -> bool {
    event.token() == self.token()
  }
}

// something that can be (de)registered on a selector
// how actual i/o happens is not specified in this trait
// use Context::register instead of these methods directly
pub trait AsyncSource {
  fn register(&mut self, selector: &mio::Poll, token: Token) -> std::io::Result<()>;
  fn deregister(&mut self, selector: &mio::Poll) -> std::io::Result<()>;
}

/// An AsyncSource that is registered on a selector with a given token
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
}

//R should not be constrained to AsyncSource here
//we want to pass the socket as an EventSource but not an AsyncSource because
//only the server should be able to deregister it
impl<R> EventSource for Registered<R> {
  fn token(&self) -> AsyncToken {
    self.token
  }
}

impl<R: Read> Read for Registered<R> {
  fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
    self.source.read(buf)
  }
}

impl<R: Write> Write for Registered<R> {
  fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
    self.source.write(buf)
  }

  fn flush(&mut self) -> std::io::Result<()> {
    self.source.flush()
  }
}

impl<R: ReadSizeHint> ReadSizeHint for Registered<R> {
  fn read_size_hint(&self) -> Option<usize> {
    self.source.read_size_hint()
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
