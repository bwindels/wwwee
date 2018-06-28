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

pub struct SocketWrapper<'a> {
  engine: &'a mut engine::Context,
  socket: &'a mut Socket
}

impl<'a> SocketWrapper<'a> {
  pub fn new(engine: &'a mut engine::Context, socket: &'a mut Socket) -> SocketWrapper<'a> {
    SocketWrapper {engine, socket}
  }

  pub fn is_writable(&self) -> bool {
    self.engine.sendapp_buf().is_some()
  }

  pub fn is_readable(&self) -> bool {
    self.engine.recvapp_buf().is_some()
  }

  /// tries to send tls records over the socket
  //returns IoReport because could interact with real socket that returns would_block
  pub fn write_records(&mut self) -> Result<IoReport> {
    self.debug_state("start write_records");
    let engine = &mut self.engine;
    let socket : &mut Write = &mut self.socket;
    engine.sendrec_buf()
      .map(|buffer| send_buffer(socket, buffer))
      .map(|write_result| {
        if let Ok(report) = write_result {
          if !report.is_empty() {
            engine.sendrec_ack(report.byte_count()) //byte_count can be 0 here, danger!
              .map_err(|err| err.as_io_error("engine error after sendrec ack"))?;
          }
        }
        write_result
      })
       //no buffer available, so no error, and wouldn't block
      .unwrap_or(Ok(IoReport::with_buffer_size(0).ok(0)))
  }
  
  //returns IoReport because could interact with real socket that returns would_block
  pub fn read_records(&mut self) -> Result<IoReport> {
    self.debug_state("start read_records");
    let socket : &mut Read = &mut self.socket;
    let engine = &mut self.engine;
    // feed records from socket into tls engine
    engine.recvrec_buf()
      .map(|buffer| {
        print!("  got read buffer of size {} ... ", buffer.len());
        receive_buffer(socket, buffer)
      })
      .map(|read_result| {
        if let Ok(report) = read_result {
          print!("read {} from socket ... ", report.byte_count());
          if !report.is_empty() {
            engine.recvrec_ack(report.byte_count())
              .map_err(|err| {
                println!("recvrec ack error: {:?}", err);
                err.as_io_error("engine error after recvrec ack")
              })?;
            println!("ack'ed!");
          }
          else {
            println!("no socket bytes :(");
          }
        }
        read_result
      })
      .unwrap_or(Ok(IoReport::with_buffer_size(0).ok(0)))
  }

  fn read_plaintext(&mut self, dst_buffer: &mut [u8]) -> Result<usize> {
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
        .map_err(|err| err.as_io_error("engine error after recvapp ack"))?;
      Ok(len)
    })
    .unwrap_or(Ok(0))
  }

  fn write_plaintext(&mut self, src_buffer: &[u8]) -> Result<usize> {
    let engine = &mut self.engine;
    //space available to send plaintext data?
    engine.sendapp_buf().map(|dst_buffer| {
      let len = cmp::min(src_buffer.len(), dst_buffer.len());
      let op_src_buffer = &src_buffer[.. len];
      let op_dst_buffer = &mut dst_buffer[.. len];
      op_dst_buffer.copy_from_slice(op_src_buffer);
      len
    })
    .map(|len| {
      engine.sendapp_ack(len)
        .map_err(|err| err.as_io_error("engine error after sendapp ack"))?;
      Ok(len)
    })
    .unwrap_or(Ok(0))
  }

  pub fn close(&mut self) -> Result<()> {
    self.engine.close();

    self.debug_state("inside close loop, about to discard app data");
    //discard incoming app data
    self.discard_incoming_data()?;
    self.debug_state("inside close loop, about to write remaining records");
    //write outstanding records
    while self.write_records()?.should_retry() {};
    
    self.engine.last_error()
      .map_err(|err| err.as_io_error("engine error while closing tls socket"))
  }

  pub fn discard_incoming_data(&mut self) -> Result<()> {
    while let Some(len) = self.engine.recvapp_buf().map(|buf| buf.len()) {
      self.engine.recvapp_ack(len)
        .map_err(|err| err.as_io_error("engine error while discarding app data"))?;
    }
    Ok( () )
  }

  pub fn is_closed(&self) -> bool {
    self.engine.is_closed()
  }

  pub fn debug_state(&self, label: &str) {
    print!("{}, TLS engine state flags: ", label);
    for f in self.engine.state() {
      print!("{:?}, ", f);
    }
    println!("");
  }
}



impl<'a> Read for SocketWrapper<'a> {

  fn read(&mut self, dst_buffer: &mut [u8]) -> Result<usize> {
    self.debug_state("SocketWrapper::read begin");
    let buffer_len = dst_buffer.len();
    let mut should_retry = true;
    //try read remaining decrypted bytes from last call to read
    let mut app_bytes_read = self.read_plaintext(dst_buffer)?;

    while should_retry && app_bytes_read != buffer_len {
      should_retry = self.read_records()?.should_retry();
      //error handling, what to do if we get an error after a few iterations here?
      //the data would be lost
      app_bytes_read += self.read_plaintext(&mut dst_buffer[app_bytes_read ..])?;
    }
    println!("SocketWrapper::read done with {:?} bytes read", app_bytes_read);
    Ok(app_bytes_read)
  }
}

impl<'a> ReadSizeHint for SocketWrapper<'a> {
  fn read_size_hint(&self) -> Option<usize> {
    self.engine.recvapp_buf().map(|b| b.len())
  }
}


impl<'a> Write for SocketWrapper<'a> {
  fn write(&mut self, src_buffer: &[u8]) -> Result<usize> {
    self.debug_state("SocketWrapper::write begin");
    let buffer_len = src_buffer.len();
    let mut should_retry = true;
    let mut app_bytes_written = 0;

    while should_retry && app_bytes_written != buffer_len {
      app_bytes_written += self.write_plaintext(&src_buffer[app_bytes_written ..])?;      
      //we fed the tls engine as much plaintext as we could
      //then we tried writing out to the socket
      //if this fails to write any bytes
      //it's best to not keep trying
      //to not end up in an endless loop.
      //also stop on would_block
      should_retry = self.write_records()?.should_retry();
    }
    println!("SocketWrapper::write done with {:?} bytes written", app_bytes_written);
    Ok(app_bytes_written)
  }

  fn flush(&mut self) -> Result<()> {
    self.engine.flush(true); //force emit non-full record
    self.write_records().map(|_| () )
  }
}

impl<'a> EventSource for SocketWrapper<'a> {
  fn token(&self) -> AsyncToken {
    self.socket.token()
  }
}
