mod ffi {

}

struct Reader<'a> {
  buffer: RefCell<Buffer<'a>>,
  read_operation: Option<RefMut<Buffer>>,
  file_fd: RawFd,
  event_fd: RawFd,
  token: AsyncToken,
  io_ctx: ffi::aio_context_t,
  next_offset: usize,
  range_end: Option<usize>
}

impl<'a> Reader<'a> {
  pub fn new(path: &Path, range: Option<Range<usize>>, buffer: Buffer<'a>, selector: &mut Poll, token: AsyncToken) -> Result<Reader> {
    let fd = libc::open(path, O_RDONLY | O_DIRECT | O_NOATIME | O_NONBLOCK)?;
    //call eventfd
    //call io_setup
    //add event_fd with token to selector, using mio::unix::EventedFd
    Reader {
      buffer: RefCell::new(buffer),
      read_operation: None,
      file_fd,
      event_fd,
      token
    }
  }
  //what error to return if borrowed?
  pub fn try_queue_read(&mut self) -> Result<()> {
    //assign borrow to read_operation, or return error if already borrowed
    //call io_submit
  }

  pub fn try_get_read_bytes<'a>(&'a mut self) -> Result<Ref<Buffer>> {
    //io_getevents
    //if obtained 1 event, remove read_operation borrow and return new borrow
  }
}
