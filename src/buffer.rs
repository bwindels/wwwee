//we could use the size_hint to pick from pools with different sizes.
//for example small errors could use a small hint and we could have a pool
//with buffers of only 256 bytes in which this response would fit.
//Then those reponse buffers would not be taken for actual correct responses created by handlers.

use std::ptr;
use std::ops::Range;
use std::cell::{RefCell, RefMut};
use std::cmp;
use std::io;
/*
trait Responder {
  fn with_buffer(size_hint: usize) -> Result<ResponseBuffer>;
  fn with_file(head_buffer: ResponseBuffer, path: &Path, range: Option<Range>) -> Result<FileResponse>;
}*/

struct Buffer<'b> {
  array: RefMut<'b, [u8]>,
  used_len: usize
}

impl<'b> Buffer<'b> {

  pub fn new(slice: RefMut<'b, [u8]>) -> Buffer {
    Buffer { array: slice, used_len: 0}
  }

  pub fn remaining(&self) -> usize {
    self.array.len() - self.used_len
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
      self.array[range].as_ptr(),
      self.array[to .. total_len].as_mut_ptr(),
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
    &self.array[.. self.used_len]
  }

  pub fn as_slice_mut<'a>(&'a mut self) -> &'a [u8] {
    &mut self.array[.. self.used_len]
  }

  pub fn write_into<R: io::Read>(&mut self, reader: &mut R) -> io::Result<usize> {
    let result = {
      let remaining_buffer = &mut self.array[self.used_len ..];
      reader.read(remaining_buffer)
    };
    if let Ok(bytes_written) = result {
      self.used_len += bytes_written;
      assert!(self.used_len <= self.array.len());
    }
    result
  }
}

impl<'b> io::Write for Buffer<'b> {
  fn write(&mut self, src: &[u8]) -> io::Result<usize> {
    let len = cmp::min(self.remaining(), src.len());
    let dst = &mut self.array[self.used_len ..];

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

type Slice4K = [u8; 4096];

struct BufferPool {
  buffers: [RefCell<Slice4K>; 10]
}

enum Error {
  Full
}

impl BufferPool {
  fn borrow_buffer<'b>(&'b mut self, _min_size: usize) -> Result<Buffer<'b>, Error> {
    for b in self.buffers.iter() {
      if let Ok(r) = b.try_borrow_mut() {
        return Ok(Buffer::new(r));
      }
    }
    return Err(Error::Full);
  }
}