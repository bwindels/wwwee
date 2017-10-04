use http::{RequestHandler, Request, BufferResponse};
 
pub struct HelloWorld {
  
}

impl HelloWorld {
  pub fn new() -> HelloWorld {
    HelloWorld {}
  }
}

impl RequestHandler for HelloWorld {
  fn read_headers(&mut self, request: &Request) -> Option<BufferResponse> {
    None
  }
  fn read_body(&mut self, body: &mut [u8]) -> Option<BufferResponse> {
    None
  }
}