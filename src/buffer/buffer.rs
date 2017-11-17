use std::ptr;
use std::ops::Range;
use std::cmp;
use std::io;
use std::ops::DerefMut;

/// guarantees that slice won't be dereferenced beyond where this buffer
/// has written, so it does not need to be cleared.
pub struct Buffer<D> {
  slice: D,
  used_len: usize
}

impl<D: DerefMut<Target=[u8]>> Buffer<D> {

  pub fn from_slice(slice: D) -> Buffer<D> {
    Buffer { slice: slice, used_len: 0}
  }

  pub fn remaining(&self) -> usize {
    self.slice.len() - self.used_len
  }

  //moves the given range to the given index and removes the rest
  pub fn keep(&mut self, range: Range<usize>, to: usize) -> usize {
    let start = cmp::min(range.start, self.used_len);
    let end = cmp::min(range.end, self.used_len);
    let range = cmp::min(start, end) .. cmp::max(start, end);
    let len = range.end - range.start;
    //don't write; the end would point past the current end,
    //exposing uninitialized data or worse
    if to > (self.used_len - len)  {
      return 0;
    }

    let total_len = to + len;

    unsafe {ptr::copy(
      self.slice[range].as_ptr(),
      self.slice[to .. total_len].as_mut_ptr(),
      len
    )};

    self.used_len = total_len;
    total_len
  }

  pub fn shrink(&mut self, new_size: usize) -> usize {
    let size = cmp::min(self.used_len, new_size);
    self.used_len = size;
    size
  }

  pub fn len(&self) -> usize {
    self.used_len
  }

  pub fn as_slice<'a>(&'a self) -> &'a [u8] {
    &self.slice[.. self.used_len]
  }

  pub fn as_mut_slice<'a>(&'a mut self) -> &'a [u8] {
    &mut self.slice[.. self.used_len]
  }

  pub fn read_from<R: io::Read>(&mut self, reader: &mut R) -> io::Result<usize> {
    let result = {
      let remaining_buffer = &mut self.slice[self.used_len ..];
      reader.read(remaining_buffer)
    };
    if let Ok(bytes_written) = result {
      self.used_len += bytes_written;
      assert!(self.used_len <= self.slice.len());
    }
    result
  }
}

impl<D: DerefMut<Target=[u8]>> io::Write for Buffer<D> {
  fn write(&mut self, src: &[u8]) -> io::Result<usize> {
    let len = cmp::min(self.remaining(), src.len());
    let dst = &mut self.slice[self.used_len ..];

    if len == 0 {
      Err(io::Error::new(io::ErrorKind::WriteZero, "Buffer is full"))
    }
    else {
      unsafe {ptr::copy_nonoverlapping(
        src.as_ptr(),
        dst.as_mut_ptr(),
        len
      )};
      self.used_len += len;
      Ok(len)
    }
  }

  fn flush(&mut self) -> io::Result<()> {
    Ok( () )
  }
}

#[cfg(test)]
mod tests {
  use super::Buffer;
  use std::io::Write;
  use std::io;

  #[test]
  fn test_write() {
    let mut array = [0u8; 40];
    let mut buffer = Buffer::from_slice(&mut array[..]);
    assert_eq!(buffer.as_slice(), b"");
    write!(buffer, "hello {}", 1).unwrap();
    assert_eq!(buffer.as_slice(), b"hello 1");
  }

  #[test]
  fn test_write_too_large() {
    let mut array = [0u8; 4];
    let mut buffer = Buffer::from_slice(&mut array[..]);
    let res = write!(buffer, "hello");
    assert!(res.is_err());
    assert_eq!(buffer.as_slice(), b"hell");
    assert_eq!(buffer.len(), 4);
    assert_eq!(buffer.remaining(), 0);
  }

  #[test]
  fn test_write_full() {
    let mut array = [0u8; 4];
    let mut buffer = Buffer::from_slice(&mut array[..]);
    write!(buffer, "hell").unwrap();
    let res = write!(buffer, "o");
    assert_eq!(res.err().map(|err| err.kind()), Some(io::ErrorKind::WriteZero));
    assert_eq!(buffer.len(), 4);
    assert_eq!(buffer.remaining(), 0);
  }

  #[test]
  fn test_len_remaining() {
    let mut array = [0u8; 10];
    let mut buffer = Buffer::from_slice(&mut array[..]);
    assert_eq!(buffer.len(), 0);
    assert_eq!(buffer.remaining(), 10);
    write!(buffer, "foo").unwrap();
    assert_eq!(buffer.len(), 3);
    assert_eq!(buffer.remaining(), 7);
  }

  #[test]
  fn test_read_from() {
    let data = b"hello world";
    let mut array = [0u8; 5];
    let mut buffer = Buffer::from_slice(&mut array[..]);
    let res = buffer.read_from(&mut data.as_ref());
    assert_eq!(res.ok(), Some(5));
    assert_eq!(buffer.remaining(), 0);
    assert_eq!(buffer.len(), 5);
    assert_eq!(buffer.as_slice(), b"hello");
  }
  
  #[test]
  fn test_shrink() {
    let mut array = [0u8; 5];
    let mut buffer = Buffer::from_slice(&mut array[..]);
    write!(buffer, "hello").unwrap();
    assert_eq!(buffer.len(), 5);
    assert_eq!(buffer.shrink(4), 4);
    assert_eq!(buffer.as_slice(), b"hell");
    assert_eq!(buffer.shrink(10), 4);
    assert_eq!(buffer.as_slice(), b"hell");
  }

  #[test]
  fn test_keep() {
    let mut array = [0u8; 20];
    let mut buffer = Buffer::from_slice(&mut array[..]);
    write!(buffer, "hello world").unwrap();
    buffer.keep(6..11, 4);
    assert_eq!(buffer.as_slice(), b"hellworld");
  }

  #[test]
  fn test_reuse_slice() {
    let mut array = [0u8; 20];
    {
      let mut buffer = Buffer::from_slice(&mut array[..]);
      write!(buffer, "hello world").unwrap();
    }
    {
      let mut buffer = Buffer::from_slice(&mut array[..]);
      assert_eq!(buffer.as_slice(), b"");
      write!(buffer, "crazy").unwrap();
    }
    assert_eq!(array.as_ref(), b"crazy world\0\0\0\0\0\0\0\0\0");
  }
}
