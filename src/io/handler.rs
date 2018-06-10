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
pub struct EventKind(usize);

const READABLE : usize = 0b01;
const WRITABLE : usize = 0b01;

impl EventKind {
  pub fn new() -> EventKind {
    EventKind(0)
  }

  pub fn with_readable(self, readable: bool) -> EventKind {
    if readable {
      EventKind(self.0 | READABLE)
    }
    else {
      self
    }
  }

  pub fn with_writable(self, writable: bool) -> EventKind {
    if writable {
      EventKind(self.0 | WRITABLE)
    }
    else {
      self
    }
  }

  pub fn is_readable(self) -> bool {
    (self.0 & READABLE) != 0
  }

  pub fn is_writable(self) -> bool {
    (self.0 & WRITABLE) != 0
  }

  pub fn has_any(self) -> bool {
    self.0 != 0
  }
}

pub trait Handler<T> {
  fn handle_event(&mut self, event: &Event, ctx: &mut Context) -> Option<T>;
}
