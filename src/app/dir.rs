use std;
use http;
use io::sources::file;

pub struct StaticDirectoryHandler<'a> {
  root_dir: &'a file::Directory,
  index_file: &'a str
}

impl<'a> StaticDirectoryHandler<'a> {
  pub fn new(root_dir: &'a file::Directory, index_file: &'a str) -> StaticDirectoryHandler<'a> {
    StaticDirectoryHandler {
      root_dir,
      index_file
    }
  }

  fn file_path(&self, request_url: &'a str) -> std::io::Result<file::RelativePath<'a, 'a>> {
    let relative_url = request_url.get(1..).and_then(|relative_url| {
      if relative_url.is_empty() {
        None
      } else {
        Some(relative_url)
      }
    });
    let path = if let Some(relative_url) = relative_url {
      if relative_url.ends_with("/") {
        self.root_dir.sub_dir_with_file(relative_url, self.index_file)
      } else {
        self.root_dir.sub_path(relative_url)
      }
    } else {
      self.root_dir.sub_path(self.index_file)
    }?;
    Ok(path)
  }
}

impl<'a> http::RequestHandler for StaticDirectoryHandler <'a> {
  fn read_headers(&mut self, request: &http::Request, res: &http::Responder) -> std::io::Result<Option<http::Response>> {
    // TODO: use content_range header
    let path = self.file_path(request.url())?;
    let (reader, _) = file::Reader::open(&path, None)?;
    let mut response = res.respond(http::status::OK)?;
    let mime_type = http::mime_type::from_path(request.url(), Some(self.index_file));
    response.set_header("Content-Type", mime_type)?;
    response.set_header_usize("Content-Length", reader.request_size()?)?;
    response.finish_with_file(reader)
  }
}

