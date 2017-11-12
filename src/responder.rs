use std::os::unix::io::RawFd;
use std::io;
use mio::net::TcpStream;

trait Responder : Drop {
  fn setup_poll(&mut self, event_loop: &mut Poll) -> io::Result<()>;
  fn writeable(&mut self, socket: &Write) -> bool;
}

struct BufferResponder<'a> {
  buffer: Buffer<'a>
  bytes_written: usize
}

impl<'a> Responder for BufferResponder<'a> {
  fn writeable(&mut self, socket: &TcpStream) -> bool {
    match socket.write(&self.buffer[ self.bytes_written .. ]) {
      Ok(bytes_written) => {
        self.bytes_written += bytes_written;
        self.bytes_written >= self.buffer.len()
      },
      Err(_) => true
    }
  }
}

struct FileResponder<'a> {
  header_responder: Option<BufferResponder<'a>>,
  file_fd: RawFd,
  range: Range<usize>,
  bytes_written: usize,
}

//needs to set the Content-Length after doing a fstat
//NO, fstat should be done elsewhere. this thing shouldn't know
//anything about http
impl<'a> FileResponder<'a> {
  pub fn new(headers: BufferResponder<'a>, path: Path, range: Range<usize>) -> io::Result<FileResponder<'a>> {
    //do open
  }
}

impl<'a> Responder for FileResponder<'a> {
  fn writeable(&mut self, socket: &TcpStream) -> bool {
    let headers_finished = self.header_responder.map(|ref headers| {
      headers.writeable(socket)
    }).unwrap_or(true);
 
   if headers_finished {
      self.header_responder = None;
      let offset = self.range.start + self.bytes_written;

      let len = self.range.end - self.range.end + self.bytes_written;
      let result = ffi::sendfile(self.file_fd, socket.as_raw_fd(), offset, len);
      let bytes_written = result.unwrap_or(0);
      self.bytes_written += bytes_written;
      
    }
  }
}