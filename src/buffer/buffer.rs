use std::io;
use std::ptr;
use std::slice;
use libc;

pub struct PageBuffer {
  page_size: usize,
  pages: usize,
  ptr: *mut u8  
}

impl PageBuffer {
  pub fn new(min_size: usize) -> PageBuffer {
    let page_size = unsafe { libc::sysconf(libc::_SC_PAGESIZE) } as usize;
    let pages = Self::pages_for_size(page_size, min_size);
    let size = pages * page_size;
    let ptr = unsafe {
      libc::mmap(
        ptr::null_mut(),
        size,
        libc::PROT_READ | libc::PROT_WRITE,
        libc::MAP_ANONYMOUS | libc::MAP_PRIVATE,
        -1, //fd
        0 //file offset
      )
    };
    if ptr == libc::MAP_FAILED {
      panic!("buffer allocation of {} pages of {} bytes (total of {} bytes) using mmap failed, with error: {}", pages, page_size, size, io::Error::last_os_error());
    }
    let ptr = ptr as *mut u8;
    PageBuffer {ptr, pages, page_size}
  }

  pub fn resize(&mut self, min_size: usize) {
    let old_size = self.page_size * self.pages;
    let pages = Self::pages_for_size(self.page_size, min_size);
    let new_size = self.page_size * pages; 
    let ptr = unsafe {
      libc::mremap(
        self.ptr as *mut libc::c_void,
        old_size,
        new_size,
        libc::MREMAP_MAYMOVE
      )
    };
    if ptr == libc::MAP_FAILED {
      panic!("buffer reallocation of {} pages of {} bytes (total of {} bytes) using mremap failed, with error: {}", pages, self.page_size, new_size, io::Error::last_os_error());
    }
    let ptr = ptr as *mut u8;
    self.ptr = ptr;
    self.pages = pages;
  }

  pub fn as_mut_slice<'a>(&'a mut self) -> &'a mut [u8] {
    unsafe { slice::from_raw_parts_mut(self.ptr, self.size()) }
  }

  pub fn as_slice<'a>(&'a self) -> &'a [u8] {
    unsafe { slice::from_raw_parts(self.ptr, self.size()) }
  }

  pub fn size(&self) -> usize {
    self.pages * self.page_size
  }

  fn pages_for_size(page_size: usize, min_size: usize) -> usize {
    let mut pages = min_size / page_size;
    if (pages * page_size) < min_size {
      pages += 1;
    }
    pages
  }
}

impl Drop for PageBuffer {
  fn drop(&mut self) {
    unsafe {
      libc::munmap(self.ptr as *mut libc::c_void, self.size())
    };
  }
}

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
