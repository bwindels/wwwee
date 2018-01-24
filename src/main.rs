#![allow(dead_code)]
mod split;
mod http;
mod app;
mod buffer;
mod io;
mod server;
mod query_connection;
#[cfg(test)]
mod test_helpers;

extern crate mio;
extern crate libc;

use query_connection::QueryConnection;
use server::Server;
use http::request_handler::Handler;

fn main() {
  let addr = "127.0.0.1:4000".parse().unwrap();
  let handler_creator = |socket| {
    QueryConnection::new(Handler::new(app::HelloWorld::new(), socket))
  };
  let mut server = Server::new(addr, handler_creator).unwrap();
  server.start().unwrap();
}
