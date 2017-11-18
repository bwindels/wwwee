#![allow(dead_code)]
mod split;
mod old;
mod http;
mod app;
mod buffer;
mod io;
mod server;
#[cfg(test)]
mod test_helpers;

extern crate mio;

fn main() {
  let addr = "127.0.0.1:4000".parse().unwrap();
  let handler_creator = || old::http::ConnectionHandler::new(app::HelloWorld::new());
  let mut server = old::http::Server::new(addr, handler_creator).unwrap();
  server.start().unwrap();
}
