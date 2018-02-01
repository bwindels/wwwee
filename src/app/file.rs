pub struct StaticFileHandler<'a> {
  root_dir: &'a str,
  download_file: bool
}

type PathString = StaticString<[u8;512]>;

impl<'a> RequestHandler for StaticFileHandler<'a> {
  fn read_headers(&mut self, request: &Request, res: &Responder) -> io::Result<Option<Reponse>> {
    
    let range = request.headers().content_range;
    let path = Path::abs_with_root(self.root_dir, request.uri);
    let reader = res.read_file(path, range);
    let content_length = reader.request_size();
    let mut response = res.respond(status::OK);
    //response.set_header(Header::ContentLength(content_length));
    response.set_header_usize("Content-Length", content_length);

    if (self.download_file) {
      response.set_header_writer("Content-Disposition", |&mut value| {
        write!(value, "attachment; filename={}", path.basename())
      })?;
    }

    response.finish_with_file(reader)
  }
}
