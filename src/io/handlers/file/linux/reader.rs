use buffer::Buffer;
use mio;
use std::io;
use std::mem;
use std::cmp;
use std::path::Path;
use std::ops::Range;
use std::os::unix::io::{RawFd, AsRawFd};
use std::os::unix::ffi::OsStrExt;
use libc;
use super::aio;
use super::owned_fd::OwnedFd;

const SWITCHING_ERROR_MSG : &'static str = "stuck on previous error";
const MAX_BUFFER_SIZE : usize = 400_000;

fn to_result(handle: libc::c_int) -> io::Result<libc::c_int> {
  if handle == -1 {
    Err(io::Error::last_os_error())
  }
  else {
    Ok(handle)
  }
}

fn stat(fd: RawFd) -> io::Result<libc::stat64> {
  let mut file_stats : libc::stat64 = unsafe { mem::zeroed() };
  let success = unsafe {
    libc::fstat64(fd, &mut file_stats as *mut libc::stat64)
  };
  to_result(success).map(|_| file_stats)
}

fn is_regular_file(file_stats: &libc::stat64) -> bool {
  (file_stats.st_mode & libc::S_IFMT) == libc::S_IFREG
}

fn chunk_count(total_size: usize, chunk_size: usize) -> usize {
  let mut chunks = total_size / chunk_size;
  if total_size % chunk_size != 0 {
    chunks += 1;
  }
  chunks
}

fn normalize_range(file_stats: &libc::stat64, range: Option<Range<usize>>) -> Range<usize> {
  let file_size = file_stats.st_size as usize;
  range.unwrap_or(0 .. file_size)
}

fn buffer_size(file_stats: &libc::stat64, range: &Range<usize>) -> usize {
  let total_size = range.end - range.start;
  const BUFFER_MAX_SIZE : usize = 400_000;
  let buffer_min_size = cmp::min(BUFFER_MAX_SIZE, total_size);
  let block_size = file_stats.st_blksize as usize;
  let blocks_per_read = chunk_count(buffer_min_size, block_size);
  blocks_per_read * block_size
}

enum ReadState {
  Ready(Buffer),
  Reading(aio::Operation),
  Switching,
}

pub struct Reader {
  read_state: ReadState,
  file_fd: OwnedFd,
  event_fd: OwnedFd,
  token: mio::Token,
  io_ctx: aio::Context,
  byte_range: Range<usize>,
  block_size: usize,
  block_index: usize
}

impl Reader {
  pub fn new(path: &Path, byte_range: Option<Range<usize>>, selector: &mut mio::Poll, token: mio::Token) -> io::Result<Reader> {
    let path_ptr = unsafe { mem::transmute::<*const u8, *const i8>(path.as_os_str().as_bytes().as_ptr()) };
    let file_fd = OwnedFd::from_raw_fd(to_result( unsafe {
      libc::open(
        path_ptr,
        libc::O_RDONLY |
        libc::O_DIRECT |
        libc::O_NOATIME |
        libc::O_NONBLOCK
      )
    } )? as RawFd);

    let file_stats = stat(file_fd.as_raw_fd())?;
    
    if !is_regular_file(&file_stats) {
      return Err(io::Error::new(io::ErrorKind::InvalidInput, "path does not represent a regular file"));
    }

    let byte_range = normalize_range(&file_stats, byte_range);
    let buffer = Buffer::page_sized_aligned(buffer_size(&file_stats, &byte_range));
    let block_size = file_stats.st_blksize as usize;
    let block_index = byte_range.start / block_size;

    let io_ctx = aio::Context::setup(1)?;
    let event_fd = OwnedFd::from_raw_fd(to_result(unsafe { libc::eventfd(0, libc::EFD_NONBLOCK) } )? as RawFd);

    selector.register(
      &mio::unix::EventedFd(&event_fd.as_raw_fd()),
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
      block_size,
      block_index,
      byte_range
    })
  }
  //what error to return if borrowed?
  pub fn try_queue_read(&mut self) -> io::Result<()> {
    match self.read_state {
      ReadState::Ready(_) => {
        let offset = 0;
        let read_state = mem::replace(&mut self.read_state, ReadState::Switching);
        let buffer = match read_state {
          ReadState::Ready(buffer) => buffer,
          _ => unreachable!()
        };
        let mut read_op = aio::Operation::create_read(self.file_fd.as_raw_fd(), offset, buffer);
        read_op.set_event_fd(self.event_fd.as_raw_fd());
        self.io_ctx.submit([read_op.as_iocb()].as_ref())?;
        self.read_state = ReadState::Reading(read_op);
        Ok( () )
      },
      ReadState::Reading(_) => Ok( () ),
      ReadState::Switching => Err(io::Error::new(io::ErrorKind::Other, SWITCHING_ERROR_MSG))
    }
  }

  pub fn try_get_read_bytes<'a>(&'a mut self) -> io::Result<&'a mut Buffer> {
    self.finish_read()?;
    match self.read_state {
      ReadState::Ready(ref mut buffer) => Ok(buffer),
      ReadState::Reading(_) => Err(io::Error::new(io::ErrorKind::WouldBlock, "read operation has not finished yet")),
      ReadState::Switching => Err(io::Error::new(io::ErrorKind::Other, SWITCHING_ERROR_MSG))
    }
  }

  fn finish_read(&mut self) -> io::Result<()> {
    // when reading, try to finish the read operation
    if let ReadState::Reading(_) = self.read_state {
      let read_state = mem::replace(&mut self.read_state, ReadState::Switching);
      let op = match read_state {
        ReadState::Reading(op) => op,
        _ => unreachable!()
      };
      let mut event_storage = [aio::Event::default()];
      let events = self.io_ctx.get_events(1, event_storage.as_mut(), None);
      if let Some(read_event) = events.get(0) {
        let buffer = op.into_read_result(read_event)?;
        self.read_state = ReadState::Ready(buffer);
      }
      //read hasn't finished yet
      else {
        self.read_state = ReadState::Reading(op);
      }
    }
    Ok( () )
  }

  pub fn deregister(&mut self, selector: &mut mio::Poll) -> io::Result<()> {
    selector.deregister(
      &mio::unix::EventedFd(&self.event_fd.as_raw_fd())
    )
  }
}
