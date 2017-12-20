use std::io;

pub struct Buffer {
  vec: Vec<u8>
}

impl Buffer {

  pub fn new() -> Buffer {
    Buffer {vec: Vec::with_capacity(4096)}
  }

  pub fn len(&self) -> usize {
    self.vec.len()
  }

  pub fn as_slice<'a>(&'a self) -> &'a [u8] {
    self.vec.as_slice()
  }

  pub fn as_mut_slice<'a>(&'a mut self) -> &'a mut [u8] {
    self.vec.as_mut_slice()
  }

  pub fn read_from<R: io::Read>(&mut self, reader: &mut R) -> io::Result<usize> {
    reader.read_to_end(&mut self.vec)
  }
}

impl io::Write for Buffer {
  fn write(&mut self, src: &[u8]) -> io::Result<usize> {
    self.vec.write(src)
  }

  fn flush(&mut self) -> io::Result<()> {
    self.vec.flush()
  }
}

#[cfg(test)]
mod tests {
  use super::Buffer;
  use std::io::Write;
  use std::io;

  #[test]
  fn test_write() {
    let mut buffer = Buffer::new();
    assert_eq!(buffer.as_slice(), b"");
    write!(buffer, "hello {}", 1).unwrap();
    assert_eq!(buffer.as_slice(), b"hello 1");
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
