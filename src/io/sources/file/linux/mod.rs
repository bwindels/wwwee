mod ffi;
mod aio;
mod reader;
mod owned_fd;
mod readrange;
mod path;
pub use self::reader::{Reader, ContentHashFields};
pub use self::path::Directory;
pub use self::path::Path;
pub use self::path::RelativePath;
pub use self::owned_fd::OwnedFd;

use libc;
use std::io;

fn bytes_as_block_offset(byte_offset: usize, block_size: u16) -> usize {
  byte_offset / block_size as usize
}

fn bytes_as_block_count(byte_offset: usize, block_size: u16) -> usize {
  if byte_offset % (block_size as usize) != 0 {
    bytes_as_block_offset(byte_offset, block_size) + 1
  }
  else {
    bytes_as_block_offset(byte_offset, block_size)
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
