use super::token::AsyncToken;
use super::context::Context;

pub enum OperationState<T> {
  Finished(T),
  InProgress
}

pub trait Handler<T> {
  fn readable(&mut self, token: AsyncToken, ctx: &Context) -> OperationState<T>;
  fn writable(&mut self, token: AsyncToken, ctx: &Context) -> OperationState<T>;
}
