#![allow(dead_code)]
mod split;
mod old;
mod http;
mod app;
mod buffer;
mod io;
mod server;
mod query_connection;
#[cfg(test)]
mod test_helpers;

extern crate mio;

/*
    /*let buffer_pool = BufferPool::new(
      4 * 1024,
      Category {amount: CONNECTION_COUNT, size:   4 * 1014},
      Category {amount:               20, size: 400 * 1024}
    ).unwrap();*/
*/

fn main() {
  let addr = "127.0.0.1:4000".parse().unwrap();
  let handler_creator = || old::http::ConnectionHandler::new(app::HelloWorld::new());
  let mut server = old::http::Server::new(addr, handler_creator).unwrap();
  server.start().unwrap();
}
