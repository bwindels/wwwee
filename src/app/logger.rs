use http;
use std;
use libc;
use libc::c_int;

pub struct Logger<H> {
  handler: H
}

impl<H> Logger<H> {
  pub fn new(handler: H) -> Logger<H> {
    Logger { handler }
  }
}

impl<H> http::RequestHandler for Logger<H>
  where
    H: http::RequestHandler,
{
  fn read_headers(&mut self, request: &http::Request, res: &http::Responder) -> std::io::Result<Option<http::Response>> {
    let (day, mon, year, hour, min, sec) = get_date_components();
    print!("{}/{}/{} {}:{}:{}: {} {} HTTP/{} with host {:?} => ",
      day, mon, year, hour, min, sec,
      request.method(),
      request.url(),
      request.version(),
      request.headers().host);

    let response = self.handler.read_headers(request, res);
    if let Ok(Some(ref r)) = response {
      println!("{}", r.status_code());
    }
    else {
      println!("no reponse");
    }
    response
  }
}

fn get_date_components() -> (c_int, c_int, c_int, c_int, c_int, c_int) {
  unsafe {
    let time = libc::time(libc::PT_NULL as *mut libc::time_t);
    let tm = libc::localtime(&time as *const libc::time_t);
    (
      (*tm).tm_mday,
      (*tm).tm_mon + 1,
      (*tm).tm_year + 1900,
      (*tm).tm_hour,
      (*tm).tm_min,
      (*tm).tm_sec
    )
  }
}
