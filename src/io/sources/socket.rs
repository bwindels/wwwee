use mio;
use std;
use io::{AsyncSource, Token};

impl AsyncSource for mio::net::TcpStream {
  fn register(&mut self, selector: &mio::Poll, token: Token) -> std::io::Result<()> {
    selector.register(
      self,
      token.as_mio_token(), 
      mio::Ready::readable() | mio::Ready::writable(), 
      mio::PollOpt::edge()
    )
  }
  fn deregister(&mut self, selector: &mio::Poll) -> std::io::Result<()> {
    selector.deregister(self)
  }
}
