use super::token::AsyncToken;
use super::context::Context;

pub enum OperationState<T> {
  Finished(T),
  InProgress
}

impl<T> OperationState<T> {
  pub fn into_option(self) -> Option<T> {
    match self {
      OperationState::Finished(result) => Some(result),
      OperationState::InProgress => None
    }
  }
}

pub trait Handler<T> {

  fn readable(&mut self, _token: AsyncToken, _ctx: &Context) -> OperationState<T> {
    OperationState::InProgress
  }

  fn writable(&mut self, _token: AsyncToken, _ctx: &Context) -> OperationState<T> {
    OperationState::InProgress
  }

}
