use std::io;

/// trait similar to Write but assumes it already has an internal buffer
/// that can be written (read into) from R without copying.
/// Using std::io::Read and std::io::Write would require an intermediate buffer.
pub trait ReadDst {
  fn read_from(&mut self, reader: &mut io::Read) -> io::Result<usize>;
  fn read_from_with_hint(&mut self, mut reader: &mut ReadSizeHint) -> io::Result<usize> {
    self.read_from(&mut reader)
  }
}
/// Provides a hint how many bytes can read from this source
/// This can be used to optimize allocation before reading
pub trait ReadSizeHint : io::Read {
  fn read_size_hint(&self) -> Option<usize>;
}
//default implementation
impl<T: io::Read> ReadSizeHint for T {
  fn read_size_hint(&self) -> Option<usize> {
    None
  }
}

/// trait similar to Read but assumes it already has an internal buffer that
/// can be writen to W without copying
/// Using std::io::Read and std::io::Write would require an intermediate buffer.
pub trait WriteSrc {
  fn write_to(&mut self, writer: &mut io::Write) -> io::Result<usize>;
}
