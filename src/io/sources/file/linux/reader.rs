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
use io::{AsyncSource, EventKind, Token};
use super::owned_fd::OwnedFd;
use super::requestrange::{ReadRangeConfig, ReadRange};
use super::{aio, bytes_as_block_count, to_result};
// ~400Kb, good size determined through testing ext4 and xfs
const BUFFER_MAX_SIZE : usize = 400_000;

enum OperationState {
  NotStarted(ReadRangeConfig, Buffer),
  Ready(ReadRange, Buffer),
  Reading(ReadRange, aio::Operation)
}

pub struct Reader {
  state: Option<OperationState>,
  file_fd: OwnedFd,
  event_fd: OwnedFd,
  io_ctx: aio::Context
}

impl Reader {
  pub fn open(
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
    let path_ptr = unsafe { mem::transmute::<*const u8, *const libc::c_char>(path.as_os_str().as_bytes().as_ptr()) };
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
    let buffer_block_capacity = buffer_block_size(&file_stats, &range, buffer_size_hint);
    let block_size = file_stats.st_blksize as usize;
    let buffer = Buffer::page_sized_aligned(buffer_block_capacity * block_size);
    let range_cfg = ReadRangeConfig::new(
      range, block_size as u16,
      buffer_block_capacity as u16);

    let io_ctx = aio::Context::setup(1)?;
    let event_fd = OwnedFd::from_raw_fd(to_result(unsafe {
      libc::eventfd(0, libc::EFD_NONBLOCK)
    } )? as RawFd);

    Ok(Reader {
      state: Some(OperationState::NotStarted(range_cfg, buffer)),
      io_ctx,
      file_fd,
      event_fd
    })
  }

  /// returns: bool: whether the end hasn't been
  ///   reached yet and a new operation was queued
  pub fn try_queue_read(&mut self) -> io::Result<bool> {
    let state = self.state.take();
    let new_state = match state {
      Some(OperationState::NotStarted(range_cfg, buffer)) => {
        range_cfg.first_range().map(|r| {
          let op = self.queue_read_operation(r.operation_range(), buffer)?;
          Ok(OperationState::Reading(r, op))
        })
      },
      Some(OperationState::Ready(range, buffer)) => {
        range.next().map(|r| {
          let op = self.queue_read_operation(r.operation_range(), buffer)?;
          Ok(OperationState::Reading(r, op))
        })
      },
      Some(s) => Some(Ok(s)),
      None => None
    };
    self.assign_state_or_err(new_state)
      .map(|_| self.state.is_some()) //is some when not eof
  }

  pub fn try_get_read_bytes<'a>(&'a mut self) -> io::Result<&'a mut [u8]> {
    self.finish_read()?;
    match self.state {
      Some(OperationState::Ready(ref range, ref mut buffer)) => {
        Ok(&mut buffer.as_mut_slice()[range.buffer_range()])
      },
      Some(OperationState::Reading(_, _)) => {
        Err(io::Error::new(io::ErrorKind::WouldBlock, "read has not finished yet"))
      },
      Some(OperationState::NotStarted(_, _)) => {
        Err(io::Error::new(io::ErrorKind::Other, "no read was queued")) 
      }
      None => Err(io::Error::new(io::ErrorKind::Other, "previous error or eof"))
    }
  }

  /// returns an error after eof or a previous error was
  /// returned from try_queue_read or try_get_read_bytes
  pub fn request_size(&self) -> io::Result<usize> {
    match self.state {
      Some(OperationState::Ready(ref range, _)) |
      Some(OperationState::Reading(ref range, _)) => {
        let r = range.total_range();
        Ok(r.end - r.start)
      },
      Some(OperationState::NotStarted(ref range_cfg, _)) => {
        let r = range_cfg.total_range();
        Ok(r.end - r.start)
      }
      None => Err(io::Error::new(io::ErrorKind::Other, "previous error or eof"))
    }
  }

  #[cfg(test)]
  pub fn block_size(&self) -> Option<usize> {
    match self.state {
      Some(OperationState::NotStarted(ref range_cfg, _)) => {
        Some(range_cfg.block_size())
      }
      _ => None
    }
  }

  fn queue_read_operation(&self, block_aligned_range: Range<usize>, buffer: Buffer) -> io::Result<aio::Operation> {
    let mut read_op = aio::Operation::create_read(
      self.file_fd.as_raw_fd(),
      block_aligned_range,
      buffer
    );
    read_op.set_event_fd(self.event_fd.as_raw_fd());
    self.io_ctx.submit([read_op.as_iocb()].as_ref())?;
    Ok(read_op)
  }

  fn finish_read(&mut self) -> io::Result<()> {
    let state = self.state.take();
    let new_state = match state {
      Some(OperationState::Reading(range, op)) => {
        let mut event_storage = [aio::Event::default()];
        let events = self.io_ctx.get_events(1, event_storage.as_mut(), None);
        let event_result = events.get(0)
          .ok_or(io::Error::new(io::ErrorKind::Other, "no aio event"));
        Some(event_result.and_then(|read_event| {
          let buffer = op.into_read_result(read_event)?;
          Ok(OperationState::Ready(range, buffer))
        }))
      },
      Some(s) => Some(Ok(s)),
      None => None
    };
    self.assign_state_or_err(new_state)
  }

  fn assign_state_or_err(&mut self, new_state: Option<io::Result<OperationState>>) -> io::Result<()> {
    if let Some(s) = new_state {
      self.state = Some(s?);
    }
    else {
      self.state = None;
    }
    Ok( () )
  }
}

impl AsyncSource for Reader {
  fn register(&mut self, selector: &mio::Poll, token: Token) -> io::Result<()> {
    selector.register(
      &mio::unix::EventedFd(&self.event_fd.as_raw_fd()),
      token.as_mio_token(),
      mio::Ready::readable(),
      mio::PollOpt::edge()
    )
  }

  fn deregister(&mut self, selector: &mio::Poll) -> io::Result<()> {
    selector.deregister(
      &mio::unix::EventedFd(&self.event_fd.as_raw_fd())
    )
  }

  fn is_registered_event_kind(&self, event_kind: EventKind) -> bool {
    event_kind.is_readable()
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

fn normalize_range(file_stats: &libc::stat64, range: Option<Range<usize>>) -> Range<usize> {
  let file_size = file_stats.st_size as usize;
  let range = range.unwrap_or(0 .. file_size);
  let start = cmp::min(range.start, file_size - 1);
  let end = cmp::min(range.end, file_size);
  start .. end
}

fn buffer_block_size(file_stats: &libc::stat64, range: &Range<usize>, buffer_size_hint: usize) -> usize {
  let total_size = range.end - range.start;
  let buffer_min_size = cmp::min(buffer_size_hint, total_size);
  let block_size = file_stats.st_blksize as usize;
  bytes_as_block_count(buffer_min_size, block_size as u16)
}


#[cfg(test)]
mod tests {
  use super::Reader;
  use self::helpers::*;
  use io::AsyncSource;

  const SMALL_MSG : &'static [u8] =
    include_bytes!("../../../../../test_fixtures/aio/small.txt");

  #[test]
  fn test_small_read_all() {
    let path = fixture_path("aio/small.txt\0").unwrap();
    let mut reader = Reader::new_with_buffer_size_hint(
      path.as_path(),
      None,
      100
    ).unwrap();

    assert_eq!(reader.request_size().unwrap(), SMALL_MSG.len());
    {
      let read_bytes = read_single(&mut reader);
      assert_eq!(read_bytes, SMALL_MSG);
    }
    assert_eq!(reader.try_queue_read().ok(), Some(false)); //EOF
  }

  #[test]
  fn test_small_read_range_too_big() {
    let path = fixture_path("aio/small.txt\0").unwrap();
    let reader = Reader::new_with_buffer_size_hint(
      path.as_path(),
      Some(0 .. 100),
      100
    ).unwrap();

    let mut counter = 0;
    assert_eq!(reader.request_size().unwrap(), SMALL_MSG.len());
    read_until_end(reader, |bytes| {
      assert_eq!(bytes, SMALL_MSG);
      counter += 1;
    });
    assert_eq!(counter, 1);
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

    assert_eq!(reader.request_size().unwrap(), msg.len());
    let read_bytes = read_single(&mut reader);
    assert_eq!(read_bytes, msg);
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
  fn test_u16_inc_buffer_same_size_within_request() {
    let path = fixture_path("aio/u16-inc-small.bin\0").unwrap();
    let mut reader = Reader::new_with_buffer_size_hint(
      path.as_path(),
      None,
      100
    ).unwrap();
    let (mut events, mut poll) = setup_event_loop(&mut reader);
    while reader.try_queue_read().unwrap() {
      //wait for read operation to finish
      poll.poll(&mut events, None).unwrap();
      
      let first_len = reader.try_get_read_bytes().unwrap().len();
      let second_len = reader.try_get_read_bytes().unwrap().len();
      assert_eq!(first_len, second_len);
    }
    reader.deregister(&mut poll).unwrap();
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

  #[test]
  fn test_2blocks_read_all() {
    let path = fixture_path("aio/2-blocks-one.bin\0").unwrap();
    let reader = Reader::new_with_buffer_size_hint(
      path.as_path(),
      None,
      1
    ).unwrap();

    let block_size = reader.block_size().unwrap();
    let mut request_counter = 0usize;
    read_until_end(reader, |read_bytes| {
      request_counter += 1;
      assert_eq!(read_bytes.len(), block_size);
    });
    assert_eq!(request_counter, 2);
  }

  mod helpers {
    use super::super::Reader;
    use std::env;
    use std::mem;
    use std::path::PathBuf;
    use mio;
    use io::{AsyncSource, Token};

    pub fn fixture_path(fixture_path: &str) -> Result<PathBuf, env::VarError> {
      let project_dir = env::var("CARGO_MANIFEST_DIR")?;
      let mut path = PathBuf::from(project_dir);
      path.push("test_fixtures");
      path.push(fixture_path);
      Ok(path)
    }

    pub fn setup_event_loop(reader: &mut Reader) -> (mio::Events, mio::Poll) {
      let mut poll = mio::Poll::new().unwrap();
      let token = Token::from_mio_token(mio::Token(1));
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
