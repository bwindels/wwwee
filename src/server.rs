
use mio::{Poll, Event, Events, Token, Ready, PollOpt};
use mio::net::{TcpListener, TcpStream};
use std::net::SocketAddr;
use std::io;
use std::mem;
use io::{
  OperationState,
  Handler,
  Context,
  create_token,
  split_token,
  ConnectionId,
  AsyncToken};
use buffer::pool::BufferPool;

/*
    /*let buffer_pool = BufferPool::new(
      4 * 1024,
      Category {amount: CONNECTION_COUNT, size:   4 * 1014},
      Category {amount:               20, size: 400 * 1024}
    ).unwrap();*/
*/

pub const CONNECTION_COUNT : usize = 100;
const CONN_SOCKET_TOKEN : AsyncToken = 0;
const SERVER_TOKEN : Token = Token(0);

fn initialize_connections<T>() -> [Option<T>; CONNECTION_COUNT] {
  let mut connections : [Option<T>; CONNECTION_COUNT] = 
    unsafe { mem::uninitialized() };
  
  for conn in connections.iter_mut() {
    *conn = None;
  }
  connections
}

pub struct Server<'a, T, F> {
  connections: [Option<T>; CONNECTION_COUNT],
  poll: Poll,
  server_socket: TcpListener,
  handler_creator: F,
  buffer_pool: &'a BufferPool<'a>
}

impl<'a, T, F> Server<'a, T, F> where T: Handler<()>, F: Fn(TcpStream) -> T {
  pub fn new(addr: SocketAddr, handler_creator: F, buffer_pool: &'a BufferPool<'a>) -> io::Result<Server<'a, T, F>> {
    let server_socket = TcpListener::bind(&addr)?;
    let poll = Poll::new()?;
    // Start listening for incoming connections
    poll.register(&server_socket, SERVER_TOKEN, Ready::readable(),
      PollOpt::edge())?;

    let connections = initialize_connections();

    Ok(Server {
      buffer_pool,
      connections,
      poll,
      server_socket,
      handler_creator
    })
  }

  pub fn start(&mut self) -> io::Result<()> {
    let mut events = Events::with_capacity(self.connections.len());
    loop {
      self.poll.poll(&mut events, None)?;

      for event in events.iter() {
        if event.token() == SERVER_TOKEN {
          if let Ok((socket, _)) = self.server_socket.accept() {
            self.register_connection(socket);
          }
        }
        else {
          if let Some(conn_idx) = self.handle_event(&event) {
            self.connections[conn_idx] = None;
          }
        }
      }
    }
  }

  fn handle_event(&mut self, event: &Event) -> Option<usize> {
    let (conn_id, async_token) = split_token(event.token().0);
    let conn_idx = (conn_id - 1) as usize;
    if let Some(ref mut handler) = self.connections[conn_idx] {
      let ctx = Context::new(&self.poll, self.buffer_pool, conn_id);
      let r = event.readiness();
      if r.is_readable() {
        if let OperationState::Finished(_) = handler.readable(async_token, &ctx) {
          return Some(conn_idx);
        }
      }
      if r.is_writable() {
        if let OperationState::Finished(_) = handler.writable(async_token, &ctx) {
          return Some(conn_idx);
        }
      }
    }
    None
  }

  fn register_connection(&mut self, socket: TcpStream) {
    if let Some(conn_idx) = self.connections
        .iter()
        .position(|conn| conn.is_none())
    {
      let conn_id = (conn_idx + 1) as ConnectionId;
      let token = Token(create_token(conn_id, CONN_SOCKET_TOKEN));
      
      let registration_success = self.poll.register(
        &socket,
        token, 
        Ready::readable() | Ready::writable(), 
        PollOpt::edge()
      ).is_ok();

      if registration_success {
        self.connections[conn_idx] = 
          Some((self.handler_creator)(socket));
        println!("added new connection with id {}", conn_id);
      }
    }
    else {
      println!("too many connections, dropping this one");
    }
  }
}
