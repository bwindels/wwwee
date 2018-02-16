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

pub const GIT_HASH : &'static str = env!("GIT_HASH");

fn main() {
  #[cfg(debug_assertions)]
  set_dump_core_on_panic();

  let addr = "0.0.0.0:8080".parse().unwrap();
  let handler_creator = |socket| {
    let index_handler = app::StaticFileHandler::new("./www/index.html\0", "text/html", None);
    let big_file = app::StaticFileHandler::new("./www/bigfile.img\0", "application/octet-stream", Some("raspbian.img"));
    let hello_world = app::HelloWorld::new();
    let router = app::Router::new(index_handler, hello_world, big_file);
    let logger = app::Logger::new(router);
    QueryConnection::new(Handler::new(logger, socket))
    //QueryConnection::new(Handler::new(app::StaticFileHandler::new(), socket))
  };
  let mut server = Server::new(addr, handler_creator).unwrap();
  println!("server version {} running ...", GIT_HASH);
  server.start().unwrap();
}

#[cfg(debug_assertions)]
fn set_dump_core_on_panic() {
  let prev_hook = std::panic::take_hook();

  std::panic::set_hook(Box::new(move |panic_info| {
    prev_hook(panic_info);
    let pid = unsafe { libc::getpid() };
    println!("pid {}, build git hash: {}", pid, GIT_HASH);
    unsafe { libc::kill(pid, libc::SIGABRT) };
  }));
}
