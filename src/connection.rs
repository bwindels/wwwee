use mio::{Poll, Ready, Token, PollOpt, Event};
use mio::net::{TcpListener, TcpStream};
use std::io::{Read, Write};
use std::io;
use std;

pub trait ConnectionHandler {
  fn bytes_available(&mut self, bytes: &mut [u8], stream: &TcpStream) -> usize;
}

pub struct Connection<T> {
  socket: TcpStream,
  token: Token,
  read_buffer: [u8; 4096],
  bytes_read: usize,
  handler: T
}

impl<T> Connection<T> {
  pub fn new(socket: TcpStream, token: Token) -> Connection {
    Connection {
      socket,
      token,
      read_buffer: [0; 4096],
      ready_to_write: false,
      read_request: false
    }
  }

  pub fn token(&self) -> Token {
    self.token
  }

  pub fn register(&mut self, poll: &Poll) -> io::Result<()> {
    poll.register(&self.socket, self.token,
      Ready::readable() | Ready::writable(), PollOpt::edge())
  }

  pub fn handle_event(&mut self, event: &Event, poll: &Poll) {
    if event.readiness().is_readable() {
      if let Ok(bytes_read) = self.socket.read(&mut self.read_buffer) {
        let subslice = &self.read_buffer[0 .. bytes_read];
        if let Ok(txt) = std::str::from_utf8(subslice) {
          println!("received {:?}", txt);
        }
        if bytes_read < self.read_buffer.len() {
          self.read_request = true;
        }
      }
    }
    if event.readiness().is_writable() {
      
    }

    if self.ready_to_write && self.read_request {
      let buf = b"HTTP/1.1 200 OK\r\n\r\nHi there";
      self.socket.write(buf);
      return true;
    }
    return false;
  }
}