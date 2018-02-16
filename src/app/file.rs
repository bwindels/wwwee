use http;
use io::sources::file;
use std;
use std::io::Write;

pub struct StaticFileHandler<'a> {
  path: &'a str,
  content_type: &'a str,
  download_as: Option<&'a str>
}

impl<'a> StaticFileHandler<'a> {
  pub fn new(path: &'a str, content_type: &'a str, download_as: Option<&'a str>) -> StaticFileHandler<'a> {
    StaticFileHandler {
      path,
      content_type,
      download_as
    }
  }
}

//type PathString = StaticString<[u8;512]>;

impl<'a> http::RequestHandler for StaticFileHandler <'a> {
  fn read_headers(&mut self, _request: &http::Request, res: &http::Responder) -> std::io::Result<Option<http::Response>> {
    
    //let range = request.headers().content_range;
    //let path = Path::abs_with_root(self.root_dir, request.uri);
    //let path = std::path::Path::new("/home/bwindels/dev/wwwee/test_fixtures/aio/small.txt\0");
    let path = std::path::Path::new(self.path);
    let reader = file::Reader::open(path, None)?;
    let mut response = res.respond(http::status::OK)?;
    //response.set_header(Header::ContentLength(content_length));
    response.set_header("Content-Type", self.content_type)?;
    response.set_header_usize("Content-Length", reader.request_size()?)?;

    if let Some(filename) = self.download_as {
      response.set_header_writer("Content-Disposition", |ref mut value| {
        write!(value, "attachment; filename={}", filename)
      })?;
    }

    response.finish_with_file(reader)
  }
}
