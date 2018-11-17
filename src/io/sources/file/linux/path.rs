use super::{to_result, OwnedFd};
use libc;
use std::io::{Result, Error, ErrorKind, Write};
use std::os::unix::io::{RawFd, AsRawFd};

// not related to linux/limits.h value,
// just the largest relative path we expect
const PATH_MAX : usize = 1024;

pub trait Path {
  fn open(&self, flags: libc::c_int) -> Result<OwnedFd>;
}

pub struct Directory {
    dir_fd: OwnedFd
}

impl Directory {
  // dir_path needs to be terminated with NUL
  pub fn open(dir_path: &str) -> Result<Directory> {
    let mut path_buffer : [u8; PATH_MAX] = unsafe {
      std::mem::uninitialized()
    };
    {
      let mut path_writer : &mut [u8] = &mut path_buffer;
      path_writer.write(dir_path.as_bytes())?;
      //append NUL byte
      path_writer.write(&[0u8])?;
    }

    let raw_fd = to_result( unsafe {
      libc::open(path_buffer.as_ptr() as *const i8, libc::O_DIRECTORY | libc::O_PATH)
    })?;
    Ok(Directory {
      dir_fd: OwnedFd::from_raw_fd(raw_fd)
    })
  }

  // to support a directory with index.html
  // http handler would use this when path ends with / and pass "index.html" as file
  // assumes/checks sub_dir ends with /
  pub fn sub_dir_with_file<'d, 'p>(&'d self, sub_dir: &'p str, filename: &'p str) -> Result<RelativePath<'d, 'p>> {
    RelativePath::new(&self, sub_dir, Some(filename))
  }

  pub fn sub_path<'d, 'p>(&'d self, relative_path: &'p str) -> Result<RelativePath<'d, 'p>> {
    RelativePath::new(&self, relative_path, None)
  }
}

impl AsRawFd for Directory {
  fn as_raw_fd(&self) -> RawFd {
    self.dir_fd.as_raw_fd()
  }
}

pub struct RelativePath<'d, 'p> {
    base_dir: &'d Directory,
    relative_path: &'p str,
    filename: Option<&'p str>,
}

impl<'d, 'p> RelativePath<'d, 'p> {
  fn new(base_dir: &'d Directory, relative_path: &'p str, filename: Option<&'p str>) -> Result<RelativePath<'d, 'p>> {
    if let Some(dir_filename) = filename {
      if !self::path_checks::is_safe_linux_filename(dir_filename) {
        return Err(Error::new(ErrorKind::InvalidInput, "relative directory filename contained ., .. or NUL"));
      }
    }
    if !self::path_checks::is_safe_linux_relative_path(relative_path) {
      Err(Error::new(ErrorKind::InvalidInput, "relative path contained ., .. or NUL"))
    } else {
      Ok(RelativePath {
        base_dir: &base_dir, 
        relative_path: &relative_path,
        filename
      })
    }
  }
}

impl<'d, 'p> Path for RelativePath<'d, 'p> {
  fn open(&self, flags: libc::c_int) -> Result<OwnedFd> {
    let mut path_buffer : [u8; PATH_MAX] = unsafe {
      std::mem::uninitialized()
    };
    {
      let mut path_writer : &mut [u8] = &mut path_buffer;
      path_writer.write(self.relative_path.as_bytes())?;
      if let Some(filename) = self.filename {
        path_writer.write(filename.as_bytes())?;
      }
      //append NUL byte
      path_writer.write(&[0u8])?;
    }
    let raw_fd = to_result( unsafe {
      libc::openat(
        self.base_dir.as_raw_fd(),
        path_buffer.as_ptr() as *const i8,
        flags
      )
    } )?;
    Ok(OwnedFd::from_raw_fd(raw_fd))
  }
}


mod path_checks {

  const SLASH: u8 = 0x2Fu8; 
  const DOT: u8 = 0x2Eu8;
  const NUL: u8 = 0u8;

  pub fn is_safe_linux_filename(filename: &str) -> bool {
    let slice = filename.as_bytes();
    // empty filename is not supported
    if slice.is_empty() {
      return false;
    }
    // doesn't contain NUL or /
    if slice.iter().any(|b| *b == NUL || *b == SLASH) {
      return false;
    }
    // not . or ..
    slice != &[DOT] && slice != &[DOT, DOT]
  }

  pub fn is_safe_linux_relative_path(path: &str) -> bool {
    let slice = path.as_bytes();
    // empty path is not supported by linux
    // also makes assumptions below easier
    if slice.is_empty() {
      return false;
    }
    // starts with a /, so absolute path?
    if slice.first() == Some(&SLASH) {
      return false;
    }
    // any NUL byte that would truncate the string?
    if slice.iter().any(|b| *b == NUL) {
      return false;
    }
    // any . or .. components in the path?
    let contains_dot_component = slice.split(|b| *b == SLASH).any(|component| {
      component == &[DOT] || component == &[DOT, DOT]
    });
    return !contains_dot_component;
  }

  #[cfg(test)]
  mod tests {
    use super::{is_safe_linux_relative_path, is_safe_linux_filename};

    #[test]
    fn test_is_safe_linux_relative_path_negative() {
      assert!(!is_safe_linux_relative_path(""));
      assert!(!is_safe_linux_relative_path("."));
      assert!(!is_safe_linux_relative_path(".."));
      assert!(!is_safe_linux_relative_path("some/."));
      assert!(!is_safe_linux_relative_path("some/.."));
      assert!(!is_safe_linux_relative_path("some/./path"));
      assert!(!is_safe_linux_relative_path("some/../path"));
      assert!(!is_safe_linux_relative_path("some\0str"));
      assert!(!is_safe_linux_relative_path("\0some str"));
      assert!(!is_safe_linux_relative_path("some str\0"));
      assert!(!is_safe_linux_relative_path("/absolute"));
    }

    #[test]
    fn test_is_safe_linux_relative_path_positive() {
      assert!(is_safe_linux_relative_path("path"));
      assert!(is_safe_linux_relative_path("..path"));
      assert!(is_safe_linux_relative_path("some/..path"));
      assert!(is_safe_linux_relative_path("some/file.bin"));
      assert!(is_safe_linux_relative_path("..."));
      assert!(is_safe_linux_relative_path("some/path"));
    }

    #[test]
    fn test_is_safe_linux_filename_negative() {
      assert!(!is_safe_linux_filename(""));
      assert!(!is_safe_linux_filename("."));
      assert!(!is_safe_linux_filename(".."));
      assert!(!is_safe_linux_filename("some/"));
      assert!(!is_safe_linux_filename("/some"));
      assert!(!is_safe_linux_filename("some/some"));
      assert!(!is_safe_linux_filename("some\0str"));
      assert!(!is_safe_linux_filename("\0some str"));
      assert!(!is_safe_linux_filename("some str\0"));
    }

    #[test]
    fn test_is_safe_linux_filename_positive() {
      assert!(is_safe_linux_filename("path"));
      assert!(is_safe_linux_filename("..path"));
      assert!(is_safe_linux_filename("file.bin"));
      assert!(is_safe_linux_filename("..."));
    }
  }
}

