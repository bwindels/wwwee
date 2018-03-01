use super::{Context, AsyncToken};

pub struct Event {
  token: AsyncToken,
  kind: EventKind
}

impl Event {
  pub fn new(token: AsyncToken, kind: EventKind) -> Event {
    Event { token, kind }
  }

  pub fn token(&self) -> AsyncToken {
    self.token
  }

  pub fn kind(&self) -> EventKind {
    self.kind
  }
}

#[derive(Clone, Copy)]
pub enum EventKind {
  Readable,
  Writable
}

impl EventKind {
  pub fn is_readable(self) -> bool {
    match self {
      EventKind::Readable => true,
      _ => false
    }
  }

  pub fn is_writable(self) -> bool {
    match self {
      EventKind::Writable => true,
      _ => false
    }
  }
}

pub trait Handler<T> {
  fn handle_event(&mut self, event: &Event, ctx: &Context) -> Option<T>;
}
