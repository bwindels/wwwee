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
// ~400Kb, good size determined through testing ext4 and xfs
const BUFFER_MAX_SIZE : usize = 400_000;

enum ReadState {
  Ready(Buffer),
  Reading(aio::Operation),
  Switching,
  EndReached
}

pub struct Reader {
  read_state: ReadState,
  file_fd: OwnedFd,
  event_fd: OwnedFd,
  io_ctx: aio::Context,
  range: Range<usize>,
  block_size: usize,
  block_index: usize
}

impl Reader {
  pub fn new(
    path: &Path,
    range: Option<Range<usize>>) -> io::Result<Reader>
  {
    Self::new_with_buffer_size_hint(path, range, BUFFER_MAX_SIZE)
  }

  pub fn new_with_buffer_size_hint(
    path: &Path,
    range: Option<Range<usize>>,
    buffer_size_hint: usize) -> io::Result<Reader>
  {
    println!("path bytes: {:?}", path.as_os_str().as_bytes());
    let path_ptr = unsafe { mem::transmute::<*const u8, *const i8>(path.as_os_str().as_bytes().as_ptr()) };
    println!("before open");
    let file_fd = OwnedFd::from_raw_fd(to_result( unsafe {
      libc::open(
        path_ptr,
        libc::O_RDONLY |
        libc::O_DIRECT |
        libc::O_NOATIME |
        libc::O_NONBLOCK
      )
    } )? as RawFd);
    println!("got past open");
    let file_stats = stat(file_fd.as_raw_fd())?;
    println!("got past stat");
    
    if !is_regular_file(&file_stats) {
      return Err(io::Error::new(io::ErrorKind::InvalidInput, "path does not represent a regular file"));
    }

    let range = normalize_range(&file_stats, range);
    let buffer = Buffer::page_sized_aligned(buffer_size(&file_stats, &range, buffer_size_hint));
    println!("got past mmap");
    let block_size = file_stats.st_blksize as usize;
    let block_index = range.start / block_size;

    let io_ctx = aio::Context::setup(1)?;
    println!("got past io_setup");
    let event_fd = OwnedFd::from_raw_fd(to_result(unsafe { libc::eventfd(0, libc::EFD_NONBLOCK) } )? as RawFd);
    println!("got past eventfd");

    Ok(Reader {
      read_state: ReadState::Ready(buffer),
      io_ctx,
      file_fd,
      event_fd,
      block_size,
      block_index,
      range
    })
  }
  /// returns: bool: whether the end hasn't been
  ///   reached yet and a new operation was queued
  pub fn try_queue_read(&mut self) -> io::Result<bool> {
    match self.read_state {
      ReadState::Ready(_) => {
        let read_state = mem::replace(&mut self.read_state, ReadState::Switching);
        let buffer = match read_state {
          ReadState::Ready(buffer) => buffer,
          _ => unreachable!()
        };
        let mut read_op = aio::Operation::create_read(
          self.file_fd.as_raw_fd(),
          self.next_offset(),
          buffer
        );
        read_op.set_event_fd(self.event_fd.as_raw_fd());
        self.io_ctx.submit([read_op.as_iocb()].as_ref())?;
        self.read_state = ReadState::Reading(read_op);
        Ok(true)
      },
      ReadState::Reading(_) => Ok(true),
      ReadState::Switching => Err(io::Error::new(io::ErrorKind::Other, SWITCHING_ERROR_MSG)),
      ReadState::EndReached => Ok(false)
    }
  }

  pub fn try_get_read_bytes<'a>(&'a mut self) -> io::Result<&'a mut [u8]> {
    self.finish_read()?;
    match self.read_state {
      ReadState::Ready(ref mut buffer) => {
        let mut offset = 0;

        let start_block_idx = self.range.start / self.block_size;
        if self.block_index == start_block_idx {
          offset = self.range.start % self.block_size;
        }

        let len = buffer.len() - offset;
        
        Ok(&mut buffer.as_mut_slice()[offset .. len])
      },
      ReadState::Reading(_) => Err(io::Error::new(io::ErrorKind::WouldBlock, "read operation has not finished yet")),
      ReadState::Switching => Err(io::Error::new(io::ErrorKind::Other, SWITCHING_ERROR_MSG)),
      ReadState::EndReached => Err(io::Error::new(io::ErrorKind::UnexpectedEof, "end of range was reached in last read operation")),
    }
  }

  pub fn register(&self, selector: &mut mio::Poll, token: mio::Token) -> io::Result<()> {
    selector.register(
      &mio::unix::EventedFd(&self.event_fd.as_raw_fd()),
      token,
      mio::Ready::readable(),
      mio::PollOpt::edge()
    )
  }

  pub fn deregister(&mut self, selector: &mut mio::Poll) -> io::Result<()> {
    self.read_state = ReadState::EndReached;
    selector.deregister(
      &mio::unix::EventedFd(&self.event_fd.as_raw_fd())
    )
  }

  fn next_offset(&self) -> usize {
    self.block_size * self.block_index
  }

  fn try_move_after_read(&mut self, returned_len: usize, requested_len: usize) -> io::Result<()> {
    if returned_len == requested_len {
      self.block_index += returned_len / self.block_size;
      Ok( () )
    }
    else {
      let total_requested_len = (self.block_index * self.block_size) + requested_len;
      if total_requested_len >= self.range.end {
        self.read_state = ReadState::EndReached;
        Ok( () )
      }
      else {
        Err(io::Error::new(io::ErrorKind::UnexpectedEof, "didn't receive a full buffer before end of range was reached"))
      }
    }
  }

  fn finish_read(&mut self) -> io::Result<()> {
    // when reading, try to finish the read operation
    if let ReadState::Reading(_) = self.read_state {
      let mut event_storage = [aio::Event::default()];
      let events = self.io_ctx.get_events(1, event_storage.as_mut(), None);
      if let Some(read_event) = events.get(0) {
        let op = {
          let read_state = mem::replace(&mut self.read_state, ReadState::Switching);
          match read_state {
            ReadState::Reading(op) => op,
            _ => unreachable!() //can never happen because first if
          }
        };
        let buffer = op.into_read_result(read_event)?;
        self.try_move_after_read(buffer.len(), buffer.capacity())?;
        self.read_state = ReadState::Ready(buffer);
      }
    }
    Ok( () )
  }
}

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

fn buffer_size(file_stats: &libc::stat64, range: &Range<usize>, buffer_size_hint: usize) -> usize {
  let total_size = range.end - range.start;
  let buffer_min_size = cmp::min(buffer_size_hint, total_size);
  let block_size = file_stats.st_blksize as usize;
  let blocks_per_read = chunk_count(buffer_min_size, block_size);
  blocks_per_read * block_size
}


#[cfg(test)]
mod tests {
  use std::io;
  use std::os::unix::prelude::*;
  use std::fs::File;
  use std::path::PathBuf;
  use std::io::Write;
  use super::super::ffi::memfd_create;
  use super::Reader;
  use libc;
  use mio;

  fn create_temp_file() -> io::Result<(File, PathBuf)> {
    let fd = super::to_result(
      unsafe { memfd_create(b"test\0".as_ptr() as *const i8, 0) }
    )? as RawFd;
    let pid = unsafe { libc::getpid() };
    let mut path_str = String::new();
    {
      use std::fmt::Write;
      write!(path_str, "/proc/{pid}/fd/{fd}\0", pid = pid, fd = fd).unwrap();
    }

    {
      let mut msg = String::new();
      {
        use std::fmt::Write;
        write!(msg, "opening path {}\n", path_str).unwrap();
      }
      unsafe {
        libc::write(0, msg.as_str().as_ptr() as *const libc::c_void, msg.len());
        libc::fsync(0);
      }
    }

    let path = PathBuf::from(path_str);
    let file = unsafe { File::from_raw_fd(fd) };
    Ok((file, path))
  }

  #[test]
  fn test_small_read() {
    const MSG : &'static [u8] = b"this is a small message in a file";

    let (mut file,pathbuf) = create_temp_file().unwrap();
    file.write(MSG).unwrap();

    //unsafe { libc::sleep(100); }

    let mut reader = Reader::new_with_buffer_size_hint(
      pathbuf.as_path(),
      None,
      100
    ).unwrap();

    let is_queued = reader.try_queue_read().unwrap();
    assert!(is_queued);

    let mut events = mio::Events::with_capacity(1);
    let mut poll = mio::Poll::new().unwrap();
    let token = mio::Token(1);
    reader.register(&mut poll, token).unwrap();
    //wait for read operation to finish
    poll.poll(&mut events, None).unwrap();
    
    let read_bytes = reader.try_get_read_bytes().unwrap();
    assert_eq!(read_bytes, MSG);

    file.set_len(0).unwrap(); //make sure file does not get dropped/closed prematurely
  }

}
