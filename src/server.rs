use connection::{Connection, ConnectionHandler};

use mio::*;
use mio::net::{TcpListener, TcpStream};
use std::io::{Read, Write};
use std::net::SocketAddr;
use std::io;
use std::mem;

pub struct Server<T: ConnectionHandler> {
  connections: [Option<Connection<T>>; 4],
  poll: Poll,
  server_socket: TcpListener,
  server_token: Token,
  events: Events,
  token_counter: usize
}

impl<T> Server<T> {
  pub fn new(addr: SocketAddr) -> io::Result<Server<T>> {
    let server_token = Token(0);
    let server_socket = TcpListener::bind(&addr)?;
    let poll = Poll::new()?;
    // Start listening for incoming connections
    poll.register(&server_socket, server_token, Ready::readable(),
            PollOpt::edge())?;

    let events = Events::with_capacity(6);
    let mut connections : [Option<Connection>; 2] = unsafe { mem::uninitialized() };
    for conn in connections.iter_mut() {
      *conn = None;
    }

    Ok(Server {
      addr,
      connections,
      poll,
      server_socket,
      server_token,
      events,
      token_counter: 100
    })
  }

  pub fn start(&mut self) -> io::Result<()> {
    loop {
        self.poll.poll(&mut self.events, None)?;

        for event in self.events.iter() {
          if event.token() == self.server_token {
            // Accept and drop the socket immediately, this will close
            // the socket and notify the client of the EOF.
            if let Ok((socket, _)) = self.server_socket.accept() {
              if let Some(conn_slot) = self.connections.iter_mut().find(|conn| conn.is_none()) {
                self.token_counter += 1;
                let token = Token(self.token_counter);
                let mut conn = Connection::new(socket, token);
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
              if let &mut Some(ref mut conn) = conn_slot {
                if conn.handle_event(&event, &self.poll) {
                  println!("closing connection with token {:?}", conn.token());
                  *conn_slot = None;
                }
              }
            }
          }
        }
      }
  }
}