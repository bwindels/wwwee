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

  pub fn with_kind(&self, kind: EventKind) -> Event {
    Event {token: self.token, kind}
  }
}

#[derive(Clone, Copy)]
pub struct EventKind(pub usize);

const READABLE : usize = 0b01;
const WRITABLE : usize = 0b10;

impl EventKind {
  pub fn new() -> EventKind {
    EventKind(0)
  }

  pub fn with_readable(self, readable: bool) -> EventKind {
    if readable {
      EventKind(self.0 | READABLE)
    }
    else {
      EventKind(self.0 & !READABLE)
    }
  }

  pub fn with_writable(self, writable: bool) -> EventKind {
    if writable {
      EventKind(self.0 | WRITABLE)
    }
    else {
      EventKind(self.0 & !WRITABLE)
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

#[cfg(test)]
mod tests {
  use super::EventKind;

  #[test]
  fn test_readable() {
    let no = EventKind::new().with_readable(false);
    assert!(!no.is_readable());
    assert!(!no.has_any());
    let yes = EventKind::new().with_readable(true);
    assert!(yes.is_readable());
    assert!(yes.has_any());


    let yes_then_no = EventKind::new().with_readable(true).with_readable(false);
    assert!(!yes_then_no.is_readable());
  }

  #[test]
  fn test_writable() {
    let no = EventKind::new().with_writable(false);
    assert!(!no.is_writable());
    assert!(!no.has_any());
    let yes = EventKind::new().with_writable(true);
    assert!(yes.is_writable());
    assert!(yes.has_any());

    let yes_then_no = EventKind::new().with_writable(true).with_writable(false);
    assert!(!yes_then_no.is_writable());
  }

  #[test]
  fn test_writable_readable() {
    let no = EventKind::new().with_writable(false).with_readable(false);
    assert!(!no.is_readable());
    assert!(!no.is_writable());
    assert!(!no.has_any());
  }
  
}
