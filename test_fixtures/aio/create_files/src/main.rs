use std::fs::File;
use std::mem;
use std::io::Write;
use std::io;
use std::os::unix::io::{RawFd, AsRawFd};

extern crate libc;

fn main() {
  let mut file = File::create("../small.txt").unwrap();
  write!(file, "try reading this with direct IO").unwrap();

  let mut file = File::create("../u16-inc-small.bin").unwrap();
  file.set_len(0).unwrap();
  //write 9kb of increasing u16
  for counter in 0 .. 4608u16 {
    let bytes = unsafe { mem::transmute::<u16, [u8; 2]>(counter) };
    file.write(&bytes).unwrap();
  }

  let block_size = get_fs_blocksize_for_file(&file).expect("could not get block_size");
  let mut file = File::create("../2-blocks-one.bin").unwrap();
  file.set_len(0).unwrap();
  //write 2 blocks of 1
  for _ in 0 .. (block_size * 2) {
    file.write([0xFFu8].as_ref()).unwrap();
  }

  let mut file = File::create("../u32-inc-big.bin").unwrap();
  file.set_len(0).unwrap();
  //write 1 mb of increasing u32
  for counter in 0 .. (1024u32 * 1024u32) / 4u32 {
    let bytes = unsafe { mem::transmute::<u32, [u8; 4]>(counter) };
    file.write(&bytes).unwrap();
  }
}

fn get_fs_blocksize_for_file(file: &File) -> io::Result<usize> {
  stat(file.as_raw_fd()).map(|s| s.st_blksize as usize)
}

fn stat(fd: RawFd) -> io::Result<libc::stat64> {
  let mut file_stats : libc::stat64 = unsafe { mem::zeroed() };
  let success = unsafe {
    libc::fstat64(fd, &mut file_stats as *mut libc::stat64)
  };
  to_result(success).map(|_| file_stats)
}


fn to_result(handle: libc::c_int) -> io::Result<libc::c_int> {
  if handle == -1 {
    Err(io::Error::last_os_error())
  }
  else {
    Ok(handle)
  }
}
