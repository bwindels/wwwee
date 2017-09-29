#![allow(dead_code)]
mod error;
mod str;
mod headers;
extern crate mio;
//extern crate arrayvec;
use mio::*;
use mio::net::{TcpListener, TcpStream};
use std::io::{Read, Write};

struct Connection {
  socket: TcpStream,
  token: Token,
  read_buffer: [u8; 4096],
  readiness: Ready,
  ready_to_write: bool,
  read_request: bool
}

impl Connection {
  pub fn new(socket: TcpStream, token: Token) -> Connection {
    Connection {
      socket,
      token,
      readiness: Ready::readable() | Ready::writable(),
      read_buffer: [0; 4096],
      ready_to_write: false,
      read_request: false
    }
  }

  pub fn token(&self) -> Token {
    self.token
  }

  pub fn register(&mut self, poll: &Poll) -> std::io::Result<()> {
    poll.register(&self.socket, self.token, self.readiness, PollOpt::edge())
  }

  pub fn deregister(&mut self, poll: &Poll) -> std::io::Result<()> {
    poll.deregister(&self.socket)
  }

  pub fn handle_event(&mut self, event: &Event, poll: &Poll) -> bool {
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
      self.ready_to_write = true;
    }

    if self.ready_to_write && self.read_request {
      let buf = b"HTTP/1.1 200 OK\r\n\r\nHi there";
      self.socket.write(buf);
      return true;
    }
    return false;
  }
}

fn main() {
    
  // Setup some tokens to allow us to identify which event is
  // for which socket.
  const SERVER: Token = Token(0);

  let addr = "127.0.0.1:4000".parse().unwrap();

  // Setup the server socket
  let server = TcpListener::bind(&addr).unwrap();

  // Create a poll instance
  let poll = Poll::new().unwrap();

  // Start listening for incoming connections
  poll.register(&server, SERVER, Ready::readable(),
          PollOpt::edge()).unwrap();
  // Create storage for events
  let mut events = Events::with_capacity(100);
  let mut connections : [Option<Connection>; 2] = unsafe {std::mem::uninitialized() };
  for conn in connections.iter_mut() {
    *conn = None;
  }

  let mut token_counter = 100usize;

  loop {
    poll.poll(&mut events, None).unwrap();

    for event in events.iter() {
      if event.token() == SERVER {
        // Accept and drop the socket immediately, this will close
        // the socket and notify the client of the EOF.
        if let Ok((socket, _)) = server.accept() {
          if let Some(conn_slot) = connections.iter_mut().find(|conn| conn.is_none()) {
            token_counter += 1;
            let token = Token(token_counter);
            let mut conn = Connection::new(socket, token);
            if let Ok(_) = conn.register(&poll) {
              *conn_slot = Some(conn);
              println!("added new connection with token {:?}", token);
            }
          }
          else {
            println!("too many connections, dropping this one");
          }
        }
      }
      else {
        let conn_slot_option = connections.iter_mut().find(|conn_slot| {
          match *conn_slot {
            &mut Some(ref conn) => conn.token() == event.token(),
            _ => false
          }
        });
        if let Some(conn_slot) = conn_slot_option {
          let should_drop_conn = if let &mut Some(ref mut conn) = conn_slot {
            if conn.handle_event(&event, &poll) {
              println!("closing connection with token {:?}", conn.token());
              conn.deregister(&poll);
              true
            }
            else {
              false
            }
          }
          else {
            false
          };
          if should_drop_conn {
            *conn_slot = None;
          }
        }
      }
    }
  }

}
