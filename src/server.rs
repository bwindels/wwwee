use connection::{Connection, ConnectionHandler};

use mio::*;
use mio::net::TcpListener;
use std::net::SocketAddr;
use std::io;
use std::mem;

pub struct Server<T: ConnectionHandler, F> {
  connections: [Option<Connection<T>>; 4],
  poll: Poll,
  server_socket: TcpListener,
  server_token: Token,
  events: Events,
  token_counter: usize,
  handler_creator: F
}

impl<T, F> Server<T, F> where T: ConnectionHandler, F: Fn() -> T {
  pub fn new(addr: SocketAddr, handler_creator: F) -> io::Result<Server<T, F>> {
    let server_token = Token(0);
    let server_socket = TcpListener::bind(&addr)?;
    let poll = Poll::new()?;
    // Start listening for incoming connections
    poll.register(&server_socket, server_token, Ready::readable(),
            PollOpt::edge())?;

    let events = Events::with_capacity(6);
    let mut connections : [Option<Connection<T>>; 4] = unsafe { mem::uninitialized() };
    for conn in connections.iter_mut() {
      *conn = None;
    }

    Ok(Server {
      connections,
      poll,
      server_socket,
      server_token,
      events,
      token_counter: 100,
      handler_creator
    })
  }

  pub fn start(&mut self) -> io::Result<()> {
    loop {
        self.poll.poll(&mut self.events, None)?;

        for event in self.events.iter() {
          if event.token() == self.server_token {
            if let Ok((socket, _)) = self.server_socket.accept() {
              if let Some(conn_slot) = self.connections.iter_mut().find(|conn| conn.is_none()) {
                self.token_counter += 1;
                let token = Token(self.token_counter);
                let mut conn = Connection::new(socket, token, (self.handler_creator)());
                if let Ok(_) = conn.register(&self.poll) {
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
            let conn_slot_option = self.connections.iter_mut().find(|conn_slot| {
              match *conn_slot {
                &mut Some(ref conn) => conn.token() == event.token(),
                _ => false
              }
            });
            if let Some(conn_slot) = conn_slot_option {
              let should_remove = if let &mut Some(ref mut conn) = conn_slot {
                let should_remove = conn.handle_event(&event, &self.poll);
                if should_remove {
                  println!("closing connection with token {:?}", conn.token());
                }
                should_remove
              }
              else {false};
              if should_remove {
                *conn_slot = None;
              }
            }
          }
        }
      }
  }
}