use std::ptr;
use std::slice;
use std::io;
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
    assert_eq!(ptr as usize % page_size, 0);
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
    assert_eq!(ptr as usize % self.page_size, 0);
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
    if (pages * page_size) < min_size || pages == 0 {
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
