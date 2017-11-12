struct Context {
  poll: Poll
}

impl Context {
  fn read_file(&mut self, path: Path, range: Option<usize>, token: AsyncToken) -> io::Result<file::Reader> {

  }

  fn borrow_buffer(&mut self, min_size: usize, page_aligned: bool) -> Result<Buffer, Error> {
    
  }
}
