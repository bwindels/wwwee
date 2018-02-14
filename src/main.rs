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
  let addr = "0.0.0.0:8080".parse().unwrap();
  let handler_creator = |socket| {
    let index_handler = app::StaticFileHandler::new("./www/index.html\0", "text/html", None);
    let big_file = app::StaticFileHandler::new("./www/bigfile.zip\0", "application/zip", Some("big_file.zip"));
    let hello_world = app::HelloWorld::new();
    let router = app::Router::new(index_handler, hello_world, big_file);
    QueryConnection::new(Handler::new(router, socket))
    //QueryConnection::new(Handler::new(app::StaticFileHandler::new(), socket))
  };
  let mut server = Server::new(addr, handler_creator).unwrap();
  server.start().unwrap();
}
