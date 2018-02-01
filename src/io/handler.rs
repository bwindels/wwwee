use super::token::AsyncToken;
use super::context::Context;

// TODO: make result type an associated type, less typing in type definitions like
// Registered<Reader, usize> (would become just Registered<Reader>)
pub trait Handler<T> {

  fn readable(&mut self, _token: AsyncToken, _ctx: &Context) -> Option<T> {
    None
  }

  fn writable(&mut self, _token: AsyncToken, _ctx: &Context) -> Option<T> {
    None
  }

}
