use std;
use io;
use super::wrapper::*;
use super::handler::Handler;
use super::context::Context;

pub struct HandlerFactory<'a> {
  private_key: secret::Key<'a>,
  trust_chain: [x509::Certificate<'a>; 1],
}

impl<'a> HandlerFactory<'a> {
  
  pub fn new(x509_cert: &'a [u8], private_key: &'a [u8]) -> std::result::Result<HandlerFactory<'a>, x509::Error> {
    let cert = x509::Certificate::from_bytes(x509_cert);
    let trust_chain = [cert];
    let private_key_decoder = secret::DecoderContext::from_bytes(private_key);
    let private_key = private_key_decoder.get_key()?;
    Ok(HandlerFactory { trust_chain, private_key })
  }

  pub fn create_handler<'s, T, H: io::Handler<T>>(&'s self, child_handler: H)
    -> Handler<'s, H>
    where 'a : 's
  {
    let tls_context = Context::from_certificate(&self.trust_chain, &self.private_key).expect("could not create context");
    Handler::new(tls_context, child_handler)
  }
}
