mod ffi;
mod aio;

use buffer::Buffer;
use mio;
use std::io;
use std::mem;
use std::path::Path;
use std::ops::Range;
use std::os::unix::io::RawFd;
use std::os::unix::ffi::OsStrExt;
use libc;

fn to_result(handle: libc::c_int) -> io::Result<libc::c_int> {
  if handle == -1 {
    Err(io::Error::last_os_error())
  }
  else {
    Ok(handle)
  }
}

enum ReadState {
  Ready(Buffer),
  Reading(aio::Operation)
}

struct Reader {
  read_state: ReadState,
  file_fd: RawFd,
  event_fd: RawFd,
  token: mio::Token,



  io_ctx: aio::Context,
  range: Option<Range<usize>>
}

impl Reader {
  pub fn new(path: &Path, range: Option<Range<usize>>, mut buffer: Buffer, selector: &mut mio::Poll, token: mio::Token) -> io::Result<Reader> {
    let path_ptr = unsafe { mem::transmute::<*const u8, *const i8>(path.as_os_str().as_bytes().as_ptr()) };
    let file_fd = to_result( unsafe { libc::open(path_ptr,
      libc::O_RDONLY |
      libc::O_DIRECT |
      libc::O_NOATIME |
      libc::O_NONBLOCK) } )?;
    //let stat = libc::stat64 {};
    let offset = range.map(|r| r.start).unwrap_or(0);
    let range_end = range.map(|r| r.end);

    let io_ctx = aio::Context::setup(1)?;
    let event_fd = to_result(unsafe { libc::eventfd(0, libc::EFD_NONBLOCK) } )?;

    buffer.clear();

    selector.register(
      &mio::unix::EventedFd(&event_fd),
      token,
      mio::Ready::readable(),
      mio::PollOpt::edge()
    )?;

    Ok(Reader {
      read_state: ReadState::Ready(buffer),
      io_ctx,
      file_fd,
      event_fd,
      token,
      range
    })
  }
  //what error to return if borrowed?
  pub fn try_queue_read(&mut self) -> io::Result<()> {
    match self.read_state {
      ReadState::Reading(_) => Err(io::Error::new(io::ErrorKind::Other, "already reading")),
      ReadState::Ready(buffer) => {
        let offset = 0;
        let read_op = aio::Operation::create_read(self.file_fd, offset, buffer);
        read_op.set_event_fd(self.event_fd);
        self.io_ctx.submit([read_op.as_iocb()].as_ref())?;
        self.read_state = ReadState::Reading(read_op);
        Ok( () )
      }
    }
  }

  pub fn try_get_read_bytes<'a>(&'a mut self) -> io::Result<&'a mut Buffer> {
    self.finish_read()?;
    match self.read_state {
      ReadState::Ready(ref mut buffer) => Ok(buffer),
      ReadState::Reading(_) => Err(io::Error::new(io::ErrorKind::Other, "read operation has not finished yet"))
    }
  }

  fn finish_read(&mut self) -> io::Result<()> {
    // when reading, try to finish the read operation
    if let ReadState::Reading(ref mut op) = self.read_state {
      let event_storage = [aio::Event::default()];
      let events = self.io_ctx.get_events(1, event_storage.as_mut(), None);
      if let Some(read_event) = events.get(0) {
        let buffer = op.into_read_result(read_event)?;
        self.read_state = ReadState::Ready(buffer);
      }
    }
    Ok( () )
  }

  pub fn deregister(&mut self, selector: &mut mio::Poll) -> io::Result<()> {
    selector.deregister(
      &mio::unix::EventedFd(&self.event_fd)
    )
  }
}

impl Drop for Reader {
  fn drop(&mut self) {
    unsafe {
      libc::close(self.file_fd);
      libc::close(self.event_fd);
    }
  }
}
