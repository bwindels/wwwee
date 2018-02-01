use std::os::unix::io::{RawFd, AsRawFd};
use libc;

pub struct OwnedFd {
  fd: RawFd
}

impl OwnedFd {
  pub fn from_raw_fd(fd: RawFd) -> OwnedFd {
    OwnedFd {fd}
  }
}

impl AsRawFd for OwnedFd {
  fn as_raw_fd(&self) -> RawFd {
    self.fd
  }
}

impl Drop for OwnedFd {
  fn drop(&mut self) {
    unsafe {
      libc::close(self.fd)
    };
  }
}
