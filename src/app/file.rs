use http;
use io::sources::file;
use std;
use std::io::Write;

pub struct StaticFileHandler<'a> {
  root_dir: &'a str,
  download_file: bool
}

impl<'a> StaticFileHandler<'a> {
  pub fn new() -> StaticFileHandler<'static> {
    StaticFileHandler {root_dir: ".", download_file: true}
  }
}

//type PathString = StaticString<[u8;512]>;

impl<'a> http::RequestHandler for StaticFileHandler <'a> {
  fn read_headers(&mut self, _request: &http::Request, res: &http::Responder) -> std::io::Result<Option<http::Response>> {
    
    //let range = request.headers().content_range;
    //let path = Path::abs_with_root(self.root_dir, request.uri);
    //let path = std::path::Path::new("/home/bwindels/dev/wwwee/test_fixtures/aio/small.txt\0");
    let path = std::path::Path::new("/home/bwindels/Downloads/fotokultuur-3acb2d.zip\0");
    let reader = file::Reader::open(path, None)?;
    let mut response = res.respond(http::status::OK)?;
    //response.set_header(Header::ContentLength(content_length));
    response.set_header("Content-Type", "text/plain")?;
    response.set_header_usize("Content-Length", reader.request_size())?;

    if self.download_file {
      response.set_header_writer("Content-Disposition", |ref mut value| {
        write!(value, "attachment; filename={}", "fotokultuur-3acb2d.zip")
      })?;
    }

    response.finish_with_file(reader)
  }
}
