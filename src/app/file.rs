use std;
use http;
use io::sources::file;

pub struct StaticFileHandler<'a> {
  root_dir: &'a file::Directory,
  force_download: bool
}

impl<'a> StaticFileHandler<'a> {
  pub fn new(root_dir: &'a file::Directory, force_download: bool) -> StaticFileHandler<'a> {
    StaticFileHandler {
      root_dir,
      force_download
    }
  }
}

//type PathString = StaticString<[u8;512]>;

impl<'a> http::RequestHandler for StaticFileHandler <'a> {
  fn read_headers(&mut self, request: &http::Request, res: &http::Responder) -> std::io::Result<Option<http::Response>> {
    //let range = request.headers().content_range;
    // TODO: deal with directory paths
    // print!("{:x?}", request.url());
    let path = self.root_dir.sub_path(request.url().get(1..).unwrap())?;
    let reader = file::Reader::open(&path, None)?;
    let mut response = res.respond(http::status::OK)?;
    response.set_header("Content-Type", "text/html")?;
    response.set_header_usize("Content-Length", reader.request_size()?)?;

    // if let Some(filename) = self.download_as {
    //   response.set_header_writer("Content-Disposition", |ref mut value| {
    //     write!(value, "attachment; filename={}", filename)
    //   })?;
    // }

    response.finish_with_file(reader)
  }
}

// fn mime_type_from_extension(url: &'a )
