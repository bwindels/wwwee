#![allow(dead_code)]
mod error;
mod str;
mod connection;
mod server;
mod http;
mod app;

extern crate mio;

fn main() {
  let addr = "127.0.0.1:4000".parse().unwrap();
  let server = http::Server::<app::HelloWorld>::new(addr).unwrap();
  server.start().unwrap();
}
