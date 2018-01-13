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
  EndReached(Buffer)
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

    let range = normalize_range(&file_stats, range);
    let buffer = Buffer::page_sized_aligned(buffer_size(&file_stats, &range, buffer_size_hint));
    let block_size = file_stats.st_blksize as usize;
    let block_index = range.start / block_size;

    let io_ctx = aio::Context::setup(1)?;
    let event_fd = OwnedFd::from_raw_fd(to_result(unsafe { libc::eventfd(0, libc::EFD_NONBLOCK) } )? as RawFd);

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
          self.current_offset(),
          buffer
        );
        read_op.set_event_fd(self.event_fd.as_raw_fd());
        self.io_ctx.submit([read_op.as_iocb()].as_ref())?;
        self.read_state = ReadState::Reading(read_op);
        Ok(true)
      },
      ReadState::Reading(_) => Ok(true),
      ReadState::Switching => Err(io::Error::new(io::ErrorKind::Other, SWITCHING_ERROR_MSG)),
      ReadState::EndReached(_) => Ok(false)
    }
  }

  pub fn try_get_read_bytes<'a>(&'a mut self) -> io::Result<&'a mut [u8]> {
    let read_block_idx = self.block_index;
    self.finish_read()?;
    match self.read_state {
      ReadState::Ready(ref mut buffer) |
      ReadState::EndReached(ref mut buffer) => {
        let mut offset = 0;

        let start_block_idx = self.range.start / self.block_size;
        if read_block_idx == start_block_idx {
          offset = self.range.start % self.block_size;
        }

        let end_idx = cmp::min(buffer.len(), self.range.end - (read_block_idx * self.block_size));
        
        Ok(&mut buffer.as_mut_slice()[offset .. end_idx])
      },
      ReadState::Reading(_) => Err(io::Error::new(io::ErrorKind::WouldBlock, "read operation has not finished yet")),
      ReadState::Switching => Err(io::Error::new(io::ErrorKind::Other, SWITCHING_ERROR_MSG)),
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
    selector.deregister(
      &mio::unix::EventedFd(&self.event_fd.as_raw_fd())
    )
  }

  pub fn request_size(&self) -> usize {
    self.range.end - self.range.start
  }

  pub fn block_size(&self) -> usize {
    self.block_size
  }

  fn current_offset(&self) -> usize {
    self.block_size * self.block_index
  }

  fn next_block_index(&self, returned_len: usize, requested_len: usize) -> io::Result<Option<usize>> {
    if returned_len == requested_len {
      let next_index = self.block_index + (returned_len / self.block_size);
      Ok( Some(next_index) )
    }
    else {
      let total_requested_len = (self.block_index * self.block_size) + returned_len;
      if total_requested_len >= self.range.end {
        Ok(None)
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
        match self.next_block_index(buffer.len(), buffer.capacity())? {
          None => self.read_state = ReadState::EndReached(buffer),
          Some(idx) => {
            self.block_index = idx;
            self.read_state = ReadState::Ready(buffer)
          }
        }
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
  use super::Reader;
  use self::helpers::*;

  //contents of the small.txt fixture file
  const SMALL_MSG : &'static [u8] = b"try reading this with direct IO";

  #[test]
  fn test_small_read_all() {
    let path = fixture_path("aio/small.txt\0").unwrap();
    let mut reader = Reader::new_with_buffer_size_hint(
      path.as_path(),
      None,
      100
    ).unwrap();

    assert_eq!(reader.request_size(), SMALL_MSG.len());
    let read_bytes = read_single(&mut reader);
    assert_eq!(read_bytes, SMALL_MSG);
  }

  #[test]
  fn test_small_eof_all() {
    let path = fixture_path("aio/small.txt\0").unwrap();
    let mut reader = Reader::new_with_buffer_size_hint(
      path.as_path(),
      None,
      100
    ).unwrap();
    let (mut events, poll) = setup_event_loop(&mut reader);

    let is_queued = reader.try_queue_read().unwrap();
    assert_eq!(is_queued, true);
    poll.poll(&mut events, None).unwrap();

    reader.try_get_read_bytes().unwrap();

    let is_queued = reader.try_queue_read().unwrap();
    assert_eq!(is_queued, false);
  }

  #[test]
  fn test_small_read_range() {
    let range = 4 .. 11;
    let msg = &SMALL_MSG[range.clone()];
    let path = fixture_path("aio/small.txt\0").unwrap();
    let mut reader = Reader::new_with_buffer_size_hint(
      path.as_path(),
      Some(range.clone()),
      100
    ).unwrap();

    assert_eq!(reader.request_size(), msg.len());
    let read_bytes = read_single(&mut reader);
    assert_eq!(read_bytes, msg);
  }

  #[test]
  fn test_u16_inc_read_all() {
    let path = fixture_path("aio/u16-inc-small.bin\0").unwrap();
    let reader = Reader::new_with_buffer_size_hint(
      path.as_path(),
      None,
      100
    ).unwrap();

    let mut counter = 0u16;
    read_until_end(reader, |read_bytes| {
      for_each_u16(read_bytes, |n| {
        assert_eq!(n, counter);
        counter += 1;
      });
    });
    assert_eq!(counter, 4608);
  }

  #[test]
  fn test_u16_inc_read_range() {
    let path = fixture_path("aio/u16-inc-small.bin\0").unwrap();
    let reader = Reader::new_with_buffer_size_hint(
      path.as_path(),
      Some(1000 .. 8400),
      100
    ).unwrap();

    let mut counter = 500u16;
    let mut request_counter = 0usize;
    read_until_end(reader, |read_bytes| {
      request_counter += 1;
      for_each_u16(read_bytes, |n| {
        assert_eq!(n, counter);
        counter += 1;
      });
    });
    assert_eq!(counter, 4200);
    assert_eq!(request_counter, 3);
  }

  mod helpers {
    use super::super::Reader;
    use std::env;
    use std::mem;
    use std::path::PathBuf;
    use mio;

    pub fn fixture_path(fixture_path: &str) -> Result<PathBuf, env::VarError> {
      let project_dir = env::var("CARGO_MANIFEST_DIR")?;
      let mut path = PathBuf::from(project_dir);
      path.push("test_fixtures");
      path.push(fixture_path);
      Ok(path)
    }

    pub fn setup_event_loop(reader: &mut Reader) -> (mio::Events, mio::Poll) {
      let mut poll = mio::Poll::new().unwrap();
      let token = mio::Token(1);
      reader.register(&mut poll, token).unwrap();
      (mio::Events::with_capacity(1), poll)
    }

    pub fn read_until_end<F: FnMut(&[u8])>(mut reader: Reader, mut callback: F) {
      let (mut events, mut poll) = setup_event_loop(&mut reader);
      while reader.try_queue_read().unwrap() {
        //wait for read operation to finish
        poll.poll(&mut events, None).unwrap();
        let read_bytes = reader.try_get_read_bytes().unwrap();
        callback(read_bytes);
      }
      reader.deregister(&mut poll).unwrap();
    }

    pub fn read_single(reader: &mut Reader) -> &[u8] {
      let (mut events, poll) = setup_event_loop(reader);
      let is_queued  = reader.try_queue_read().unwrap();
      assert!(is_queued);
      poll.poll(&mut events, None).unwrap();
      let read_bytes = reader.try_get_read_bytes().unwrap();
      read_bytes    
    }

    pub fn for_each_u16<F: FnMut(u16)>(bytes: &[u8], callback: F) {
      bytes.chunks(2).map(|bytes| {
        let array = [bytes[0], bytes[1]];
        let n = unsafe { mem::transmute::<[u8;2], u16>(array) };
        n
      }).for_each(callback);
    }
  }


}
