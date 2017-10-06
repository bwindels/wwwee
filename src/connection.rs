use mio::{Poll, Ready, Token, PollOpt, Event};
use mio::net::{TcpListener, TcpStream};
use std::io::{Read, Write};
use std::io;
use std;

pub trait ConnectionHandler {
  fn bytes_available(&mut self, bytes: &mut [u8], stream: &mut TcpStream) -> usize;
}

pub struct Connection<T: ConnectionHandler> {
  socket: TcpStream,
  token: Token,
  handler: T,
  read_buffer: [u8; 4096],
  bytes_read: usize,
}

impl<T: ConnectionHandler> Connection<T> {
  pub fn new(socket: TcpStream, token: Token, handler: T) -> Connection<T> {
    Connection {
      socket,
      token,
      handler,
      read_buffer: [0; 4096],
      bytes_read: 0
    }
  }

  pub fn token(&self) -> Token {
    self.token
  }

  pub fn register(&mut self, poll: &Poll) -> io::Result<()> {
    poll.register(&self.socket, self.token,
      Ready::readable() | Ready::writable(), PollOpt::edge())
  }

  pub fn handle_event(&mut self, event: &Event, _: &Poll) -> bool {
    if event.readiness().is_readable() {
      let bytes_read = {
        let remaining_buf = &mut self.read_buffer[self.bytes_read ..];
        self.socket.read(remaining_buf)
      };
      if let Ok(bytes_read) = bytes_read {
        self.bytes_read += bytes_read;
        let num_bytes_consumed = {
          let bytes_so_far = &mut self.read_buffer[0 .. self.bytes_read];
          self.handler.bytes_available(bytes_so_far, &mut self.socket)
        };
        if num_bytes_consumed != 0 {
          let len = self.bytes_read - num_bytes_consumed;
          //remove consumed bytes from buffer, to make space for new data
          unsafe {std::ptr::copy(
            self.read_buffer[num_bytes_consumed .. self.bytes_read].as_ptr(),
            self.read_buffer[ .. len].as_mut_ptr(),
            len
          )};
          self.bytes_read = len;
        }
        return num_bytes_consumed != 0;
      }
    }
    false
    /*
        //if let Ok(txt) = std::str::from_utf8(subslice) {
    if event.readiness().is_writable() {
      
    }

    if self.ready_to_write && self.read_request {
      let buf = b"HTTP/1.1 200 OK\r\n\r\nHi there";
      self.socket.write(buf);
      return true;
    }
    return false;
    */
  }
}