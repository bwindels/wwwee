use io::{Socket, ReadSizeHint, EventSource, AsyncToken};
use io::handlers::{send_buffer, receive_buffer, IoReport};
use std::io::{Result, Error, ErrorKind, Read, Write};
use super::wrapper::engine;
use std::cmp;

/**
one the one hand we have a tcp socket, which when reading will give x amount of bytes
we need to read into the recvrec buffer until ErrorKind::WouldBlock,
after every read of the socket we need to try and decrypt the data,
and let the handler process it (append it to it's request buffer)
so we can clear some space in the recvrec and recvapp buffers.

on the other hand, every time the handler receives data from the socket
(or any other async event) it might want to send a response,
which will mean writing to the sendapp buffer. When the sendapp buffer is full,
we want to encrypt it into a tls record and send it out on the socket.
We only want to return WouldBlock when the real socket buffer is full, not when
the sendapp buffer is full, so we'll need to fill the buffer, encrypt,
send out the sendrec buffer and do that again until all the bytes from
the call to write have been sent.

so on every read to call bearssl to try and decrypt the data, because
we might have just received the end of a tls record.

on every write we also call into bearssl but on Write::flush we tell
bearssl to force a tls record. 
*/

struct SocketWrapper<'a> {
  engine: &'a mut engine::Context,
  socket: &'a mut Socket
}

impl<'a> SocketWrapper<'a> {
  pub fn new(engine: &'a mut engine::Context, socket: &'a mut Socket) -> SocketWrapper<'a> {
    SocketWrapper {engine, socket}
  }

  pub fn can_write(&self) -> bool {
    self.engine.sendapp_buf().is_some()
  }

  /// tries to send tls records over the socket
  /// returns if more writes would block
  fn try_send_records(&mut self) -> Result<bool> {
    let engine = &mut self.engine;
    let socket : &mut Write = &mut self.socket;
    engine.sendrec_buf()
      .map(|sendrec_buffer| {
        send_buffer(socket, sendrec_buffer)
      })
      .map(|result| {
        match result {
          Ok(result) => {
            engine.sendrec_ack(result.byte_count()) //byte_count can be 0 here, danger!
              .map(|_| result.is_partial())
              .map_err(|_| Error::new(ErrorKind::Other, "engine error after sendrec ack"))
          },
          Err(err) => Err(err)
        }
      })
      .unwrap_or(Ok(false)) //no buffer available, so no error, and wouldn't block
  }
}

/// READING
impl<'a> SocketWrapper<'a> {

  pub fn can_read(&self) -> bool {
    self.engine.recvapp_buf().is_some()
  }

  fn read_records(&mut self) -> Result<IoReport> {
    let socket : &mut Read = &mut self.socket;
    let engine = &mut self.engine;
    // feed records from socket into tls engine
    engine.recvrec_buf()
      .map(|buffer| receive_buffer(socket, buffer))
      .map(|read_result| {
        if let Ok(report) = read_result {
          if !report.is_empty() {
            engine.recvrec_ack(report.byte_count())
              .map_err(|_| Error::new(ErrorKind::Other, "engine error after recvrec ack"))?;
          }
        }
        read_result
      })
      .unwrap_or(Ok(IoReport::with_buffer_size(0).ok(0)))
  }

  fn read_decrypted(&mut self, dst_buffer: &mut [u8]) -> Result<usize> {
    let engine = &mut self.engine;
    // any plaintext data available?
    engine.recvapp_buf().map(|src_buffer| {
      let len = cmp::min(src_buffer.len(), dst_buffer.len());
      let op_src_buffer = &src_buffer[.. len];
      let op_dst_buffer = &mut dst_buffer[.. len];
      op_dst_buffer.copy_from_slice(op_src_buffer);
      len
    })
    .map(|len| {
      engine.recvapp_ack(len)
        .map_err(|_| Error::new(ErrorKind::Other, "engine error after recvapp ack"))?;
      Ok(len)
    })
    .unwrap_or(Ok(0))
    //read max (recvrec buffer size) bytes from socket,
      //  if read is empty, bail out with Ok(bytes_read)
      //feed into recvrec buffer
      //get recvapp buffer
      //  if recvapp buffer is not available or empty, bail out with Ok(bytes_read)
      //  copy recvapp buffer into dst_buffer
      //  increments bytes_read counter
      //  trim dst_buffer with amount of copied bytes
  }
}



impl<'a> Read for SocketWrapper<'a> {

  fn read(&mut self, dst_buffer: &mut [u8]) -> Result<usize> {
    // let socket : &mut Read = &mut self.socket;
    // let engine = &mut self.engine;
    let buffer_len = dst_buffer.len();
    let mut would_block = false;

    //try read remaining decrypted bytes from last call to read
    let mut app_bytes_read = self.read_decrypted(dst_buffer)?;

    while !would_block && app_bytes_read != buffer_len {
      let read_report = self.read_records()?;
      would_block = read_report.would_block();
      // not sure about this: if there are unconsumed bytes in 
      if read_report.is_empty() {
        return Ok(app_bytes_read)
      }
      //error handling, what to do if we get an error after a few iterations here?
      //the data would be lost
      app_bytes_read += self.read_decrypted(&mut dst_buffer[app_bytes_read ..])?;
    }
    Ok(app_bytes_read)
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
      let len = {
        let dst_buffer = self.engine.sendapp_buf()
          .ok_or(Error::new(ErrorKind::Other, "sendapp buffer not available on socket write"))?;
        
        let len = cmp::min(src_buffer.len(), dst_buffer.len());
        let src_buffer_for_write = &src_buffer[.. len];
        let dst_buffer = &mut dst_buffer[.. len];
        dst_buffer.copy_from_slice(src_buffer_for_write);
        len
      };
      self.engine.sendapp_ack(len)
        .map_err(|_| Error::new(ErrorKind::Other, "engine error after sendapp ack"))?;
      
      src_buffer = &src_buffer[len ..];
      total_bytes_written += len;
      //any records ready to be sent?
      would_block = self.try_send_records()?;
    }
    Ok(total_bytes_written)
  }

  fn flush(&mut self) -> Result<()> {
    self.engine.flush(true); //force emit non-full record
    self.try_send_records().map(|_| () )
  }
}

impl<'a> EventSource for SocketWrapper<'a> {
  fn token(&self) -> AsyncToken {
    self.socket.token()
  }
}
