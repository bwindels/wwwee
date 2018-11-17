#![allow(dead_code)]
mod split;
mod http;
mod app;
mod buffer;
mod io;
mod server;
mod query_connection;
mod tls;
#[cfg(test)]
mod test_helpers;

extern crate mio;
extern crate libc;

use query_connection::QueryConnection;
use server::Server;
use io::sources::file::Directory;
use http::request_handler::Handler;

pub const GIT_HASH : &'static str = env!("GIT_HASH");

fn main() {
  #[cfg(debug_assertions)]
  set_dump_core_on_panic();

  let tls_handler_factory = create_tls_handler_factory();
  let www_root = Directory::open("./www/").expect("./www/ dir does not exist");

  let addr = "0.0.0.0:4343".parse().unwrap();
  let handler_creator = || {
    let dir_handler = app::StaticDirectoryHandler::new(&www_root, "index.html");
    let logger = app::Logger::new(dir_handler);
    let responder = QueryConnection::new(Handler::new(logger));
    let tls_handler = tls_handler_factory.create_handler(responder);
    return tls_handler;
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

fn create_tls_handler_factory() -> tls::HandlerFactory<'static> {
  let x509_cert_bytes = include_bytes!("../conf/tls/cert.der");
  let private_key_bytes = include_bytes!("../conf/tls/private_key.der");
  tls::HandlerFactory::new(x509_cert_bytes, private_key_bytes).expect("couldn't create tls handler factory")
}
