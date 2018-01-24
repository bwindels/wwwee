type ConnectionToken = u32;
#[cfg(target_pointer_width = "64")]
type AsyncToken = u32;
#[cfg(target_pointer_width = "32")]
type AsyncToken = u8;

#[cfg(target_pointer_width = "64")]
fn split_token(token: usize) -> (ConnectionToken, AsyncToken) {
  let connection_idx = token >> 32;
  let async_handle_idx = token & 0xFFFFFFFF;
  (connection_idx, async_handle_idx)
}
#[cfg(target_pointer_width = "32")]
fn split_token(token: usize) -> (ConnectionToken, AsyncToken) {
  let connection_idx = token >> 8;
  let async_handle_idx = token & 0xFF;
  (connection_idx, async_handle_idx)
}

trait AsyncHandle : Drop {
  fn token(&self) -> AsyncToken;
}

trait AsyncFileRead<'a> : AsyncHandle {
  fn available_bytes(&mut self) -> FileBufferHandle;
}
//when dropping this, the buffer is marked as free and we queue the next chunk?
//hmm, operation that can fail should not be implicit
trait FileBufferHandle : Drop + Defer<&[u8]> {

}

struct FileReadOptions {
  pub range: Option<Range<usize>>,
  pub buffer: Option<usize>,
}

trait AwakenerCreator : Drop {
  fn create_awakener(&self) -> Awakener;
}

trait Awakener : Clone + Copy {
  fn wakeup(&self) -> io::Result<()>;
}
//a notifier local to the ConnectionToken
trait Notifier<M: Send> : Clone {
  fn notify(&self, message: M);
}
//better name? IOContext?
trait Context {
  fn read_file(&mut self, path: Path, options: FileReadOptions, token: AsyncToken) -> io::Result<AsyncFileRead>;
  fn borrow_buffer(&mut self, min_size: usize, page_aligned: bool) -> Result<Buffer, Error>;
  //fn connect(&mut self, addr: SocketAddr, token: AsyncToken, buffer: Buffer<'a>) -> io::Result<TcpStream>;
  //fn awakener(&self) -> Awakener;
  //fn notifier() -> &Notifier<M>
}

enum ConnectionState {
  Finished,
  InProgress
}

mod io {
  trait Handler {
    fn readable(&mut self, token: AsyncToken, event_loop: &EventLoop) -> ConnectionState;
    fn writeable(&mut self, token: AsyncToken, event_loop: &EventLoop) -> ConnectionState;
    //fn notify(&mut self, message: M) -> ConnectionState;
  }
}

struct FileResponder : io::Handler {
  
}

struct BufferResponder<'a> : io::Handler {
  buffer: Buffer<'a>
  bytes_written: usize
}

mod http {
  type StatusCode = u16;

  mod response {

    enum Body {
      Buffer(BufferResponder),
      File(FileResponder),
      None
    }

    pub struct Response : io::Handler {
      head: BufferResponder,
      body: Body
    }

    pub trait ResponseHead {
      fn set_header(&mut self, name: &str, value: &str) -> io::Result<()>;
      fn set_header_u64(&mut self, name: &str, value: u64) -> io::Result<()>;
      fn write_header<F: FnOnce(&mut Write) -> io::Result<usize>>(&mut self, name: &str, write_callback: F) -> io::Result<usize>;
    }

    mod file {
      pub trait ResponseHead : super::ResponseHead {
        fn finish(self) -> Response
      }
    }

    mod buffer {
      pub trait ResponseHead : super::ResponseHead {
        fn into_body(self) -> ResponseBody;
        fn finish(self) -> Response;
      }

      pub trait ResponseBody : Write {
        fn finish(self) -> Response;
      }

    }
  }

  pub trait Context {
    fn respond_with_file(&mut self, path: &Path, range: Option<Range<usize>>) -> io::Result<file::ResponseHead>;
    fn respond_with_buffer(&mut self, size: Option<usize>) -> io::Result<buffer::ResponseHead>;
  }

  pub trait RequestHandler {
    fn read_headers(&mut self, request: &Request, responder: &mut Responder) -> io::Result<Option<Response>>;  
    fn read_body(&mut self, body: &mut [u8], responder: &mut Responder) -> io::Result<Option<Response>>;
  }
}
