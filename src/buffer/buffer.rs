//we could use the size_hint to pick from pools with different sizes.
//for example small errors could use a small hint and we could have a pool
//with buffers of only 256 bytes in which this response would fit.
//Then those reponse buffers would not be taken for actual correct responses created by handlers.

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

  pub fn write_into<R: io::Read>(&mut self, reader: &mut R) -> io::Result<usize> {
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
