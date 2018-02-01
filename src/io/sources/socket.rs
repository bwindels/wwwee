use mio;
use std;
use io::{AsyncSource, EventKind, Token};

impl AsyncSource for mio::net::TcpStream {
  fn register(&mut self, selector: &mio::Poll, token: Token) -> std::io::Result<()> {
    selector.register(
      self,
      token.as_mio_token(), 
      mio::Ready::readable() | mio::Ready::writable(), 
      mio::PollOpt::edge()
    )
  }
  fn deregister(&mut self, _selector: &mio::Poll) -> std::io::Result<()> {
    Ok( () )
  }

  fn is_registered_event_kind(&self, event_kind: EventKind) -> bool {
    match event_kind {
      EventKind::Readable |
      EventKind::Writable => true
    }
  }
}
