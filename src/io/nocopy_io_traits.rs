use std::io;

/// trait similar to Write but assumes it already has an internal buffer
/// that can be written (read into) from R without copying.
/// Using std::io::Read and std::io::Write would require an intermediate buffer.
pub trait ReadDst {
  fn read_from<R: io::Read>(&mut self, reader: &mut R) -> io::Result<usize>;
  fn read_from_with_hint<R: io::Read + ReadSizeHint>(&mut self, reader: &mut R) -> io::Result<usize> {
    self.read_from(reader)
  }
}
/// Provides a hint how many bytes can read from this source
/// This can be used to optimize allocation before reading
pub trait ReadSizeHint {
  fn read_size_hint(&self) -> Option<usize>;
}

/// trait similar to Read but assumes it already has an internal buffer that
/// can be writen to W without copying
/// Using std::io::Read and std::io::Write would require an intermediate buffer.
pub trait WriteSrc {
  fn write_to<W: io::Write>(&mut self, writer: &mut W) -> io::Result<usize>;
}
