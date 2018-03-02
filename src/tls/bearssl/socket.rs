use io::{Socket, ReadSizeHint, EventSource, AsyncToken};
use io::handlers::{send_buffer, SendResult};
use std::io::{Result, Error, ErrorKind, Read, Write};
use super::wrapper::engine;
use std::cmp;

struct SocketWrapper<'a> {
  engine: &'a mut engine::Context,
  socket: &'a mut Socket
}

impl<'a> SocketWrapper<'a> {
  pub fn new(engine: &'a mut engine::Context, socket: &'a mut Socket) -> SocketWrapper<'a> {
    SocketWrapper {engine, socket}
  }

  pub fn can_read(&self) -> bool {
    self.engine.recvapp_buf().is_some()
  }

  pub fn can_write(&self) -> bool {
    self.engine.sendapp_buf().is_some()
  }

  /// tries to send tls records over the socket
  /// returns if more writes would block
  fn try_send(&mut self) -> Result<bool> {
    if let Some(sendrec_buffer) = self.engine.sendrec_buf() {
      let (would_block, bytes_written) = match send_buffer(&mut self.socket, sendrec_buffer) {
        SendResult::Consumed => {
          (false, sendrec_buffer.len())
        },
        SendResult::WouldBlock(bytes_written) => {
          (true, bytes_written)
        },
        SendResult::IoError(err) => {
          return Err(err);
        }
      };
      self.engine.sendrec_ack(bytes_written)
        .map_err(|_| Error::new(ErrorKind::Other, "engine error after sendrec ack"))?;
      Ok(would_block)
    }
    else {
      Ok(false)
    }
  }
}

impl<'a> Read for SocketWrapper<'a> {
  fn read(&mut self, dst_buffer: &mut [u8]) -> Result<usize> {
    let len = {
      let src_buffer = self.engine.recvapp_buf()
        .ok_or(Error::new(ErrorKind::WouldBlock, ""))?;
      let len = cmp::min(src_buffer.len(), dst_buffer.len());
      let src_buffer = &src_buffer[.. len];
      let dst_buffer = &mut dst_buffer[.. len];
      dst_buffer.copy_from_slice(src_buffer);
      len
    };
    self.engine.recvapp_ack(len)
      .map(|_| len)
      .map_err(|_| Error::new(ErrorKind::Other, "engine error after recvapp ack"))
  }
}

impl<'a> ReadSizeHint for SocketWrapper<'a> {
  fn read_size_hint(&self) -> Option<usize> {
    self.engine.recvapp_buf().map(|b| b.len())
  }
}


impl<'a> Write for SocketWrapper<'a> {
  fn write(&mut self, mut src_buffer: &[u8]) -> Result<usize> {
    let mut would_block = false;
    let mut total_bytes_written = 0;
    while src_buffer.len() > 0 && !would_block {
      let dst_buffer = self.engine.sendapp_buf()
        .ok_or(Error::new(ErrorKind::Other, "sendapp buffer not available on socket write"))?;
      
      let len = cmp::min(src_buffer.len(), dst_buffer.len());
      let src_buffer_for_write = &src_buffer[.. len];
      let dst_buffer = &mut dst_buffer[.. len];
      dst_buffer.copy_from_slice(src_buffer_for_write);

      self.engine.sendapp_ack(len)
        .map_err(|_| Error::new(ErrorKind::Other, "engine error after sendapp ack"))?;
      
      src_buffer = &src_buffer[len ..];
      total_bytes_written += len;
      //any records ready to be sent?
      would_block = self.try_send()?;
    }
    Ok(total_bytes_written)
  }

  fn flush(&mut self) -> Result<()> {
    self.engine.flush(true); //force emit non-full record
    self.try_send().map(|_| () )
  }
}

impl<'a> EventSource for SocketWrapper<'a> {
  fn token(&self) -> AsyncToken {
    self.socket.token()
  }
}
