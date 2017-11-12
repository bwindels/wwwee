pub struct StaticFileHandler<'a> {
  root_dir: &'a str,
  download_file: bool
}

type PathString = StaticString<[u8;512]>;

impl<'a> RequestHandler for StaticFileHandler<'a> {
  fn read_headers(&mut self, request: &Request, res: &Responder) -> io::Result<Option<Reponse>> {
    
    if let Some(path) = Path::abs_with_root(self.root_dir, request.uri) {
      if let Some(stat) = libc::stat(path) {
        let range_header = request.headers().find(Range);
        let range = range_header.map(|range_header| {

        });
        let mut response = res.respond_with_file(200, path, range);
        response.set_header_with_num("Content-Length", stat.size);
        if (self.download_file) {
          response.set_header_with_writer("Content-Disposition", |&mut value| {
            write!(value, "attachment; filename={}", path.basename())
          })?;
        }
        response.finish()
      }
    }
    res.error(NOT_FOUND).finish()
  }
}