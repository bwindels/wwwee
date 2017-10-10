#![allow(dead_code)]
mod split;
mod connection;
mod server;
mod http;
mod app;
#[cfg(test)]
mod test_helpers;

extern crate mio;

fn main() {
  let addr = "127.0.0.1:4000".parse().unwrap();
  let handler_creator = || http::ConnectionHandler::new(app::HelloWorld::new());
  let mut server = http::Server::new(addr, handler_creator).unwrap();
  server.start().unwrap();
}
