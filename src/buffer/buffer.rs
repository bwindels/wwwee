use std::io;
use std::ptr;
use super::PageBuffer;

pub struct Buffer {
  page_buffer: PageBuffer,
  len: usize
}

impl Buffer {

  pub fn new() -> Buffer {
    Buffer::page_sized_aligned(4000)
  }

  pub fn page_sized_aligned(min_size: usize) -> Buffer {
    Buffer { page_buffer: PageBuffer::new(min_size), len: 0 }
  }

  pub fn clear(&mut self) {
    self.len = 0;
  }

  pub fn len(&self) -> usize {
    self.len
  }

  pub fn capacity(&self) -> usize {
    self.page_buffer.size()
  }

  pub unsafe fn set_len(&mut self, len: usize) {
    if len <= self.capacity() {
      self.len = len;
    }
  }

  pub fn as_slice<'a>(&'a self) -> &'a [u8] {
    &self.page_buffer.as_slice()[.. self.len]
  }

  pub fn as_mut_slice<'a>(&'a mut self) -> &'a mut [u8] {
    &mut self.page_buffer.as_mut_slice()[.. self.len]
  }

  pub fn read_from<R: io::Read>(&mut self, reader: &mut R) -> io::Result<usize> {
    //TODO: read all data here from reader, not just what would fit
    //this could be optimized with an extra ReadHint trait that gives an Option<usize>
    //for the available size. This way we could only do one allocation if a lot of
    //data is available.
    let bytes_read = reader.read(self.page_buffer.as_mut_slice())?;
    self.len += bytes_read;
    Ok(bytes_read)
  }
}

impl io::Write for Buffer {
  fn write(&mut self, src: &[u8]) -> io::Result<usize> {
    let new_len = self.len + src.len();
    if new_len > self.page_buffer.size() {
      self.page_buffer.resize(new_len);
    }

    let buffer_start_ptr = self.page_buffer
      .as_mut_slice()
      .as_mut_ptr();
    
    unsafe {
      let dst_ptr = buffer_start_ptr.offset(self.len as isize);
      ptr::copy_nonoverlapping(
        src.as_ptr(),
        dst_ptr,
        src.len()
      )
    };
    self.len += src.len();
    Ok(src.len())
  }

  fn flush(&mut self) -> io::Result<()> {
    Ok( () )
  }
}

#[cfg(test)]
mod tests {
  use super::Buffer;
  use std::io::Write;

#[test]
  fn test_write() {
    let mut buffer = Buffer::new();
    assert_eq!(buffer.as_slice(), b"");
    write!(buffer, "hello").unwrap();    
    assert_eq!(buffer.as_slice(), b"hello");
  }

  #[test]
  fn test_write_appends() {
    let mut buffer = Buffer::new();
    assert_eq!(buffer.as_slice(), b"");
    write!(buffer, "hello {}", 1).unwrap();
    write!(buffer, " world {}", 2).unwrap();
    assert_eq!(buffer.as_slice(), b"hello 1 world 2");
  }

  #[test]
  fn test_clear() {
    let mut buffer = Buffer::new();
    write!(buffer, "hello").unwrap();
    buffer.clear();
    assert_eq!(buffer.as_slice(), b"");
  }

  #[test]
  fn test_grow_on_write() {
    let mut buffer = Buffer::new();
    let original_capacity = buffer.capacity();
    let times = (original_capacity / 8) + 2;
    for i in 0..times {
        write!(buffer, "{:>8}", i).unwrap();
    }
    assert!(buffer.capacity() != original_capacity);
    assert_eq!(buffer.len(), times * 8);
  }

  #[test]
  fn test_read_from() {
    let data = b"hello";
    let mut buffer = Buffer::new();
    let res = buffer.read_from(&mut data.as_ref());
    assert_eq!(res.ok(), Some(5));
    assert_eq!(buffer.len(), 5);
    assert_eq!(buffer.as_slice(), b"hello");
  }
}
