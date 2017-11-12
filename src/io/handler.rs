use ::{AsyncToken, Context};

enum OperationState<T> {
  Finished(T),
  InProgress
}

trait Handler<T> {
  fn readable(&mut self, token: AsyncToken, ctx: &mut Context) -> OperationState<T>;
  fn writeable(&mut self, token: AsyncToken, ctx: &mut Context) -> OperationState<T>;
}
