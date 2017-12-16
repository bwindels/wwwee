use super::token::AsyncToken;
use super::context::Context;

pub trait Handler<T> {

  fn readable(&mut self, _token: AsyncToken, _ctx: &Context) -> Option<T> {
    None
  }

  fn writable(&mut self, _token: AsyncToken, _ctx: &Context) -> Option<T> {
    None
  }

}
