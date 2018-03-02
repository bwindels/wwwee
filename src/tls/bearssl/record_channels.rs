use super::wrapper::{engine};
use std::io::{Read, Write, Result, Error, ErrorKind};
use io::{WriteSrc, ReadDst};

pub struct ReceiveRecordBuffer<'a> {
  engine: &'a mut engine::Context
}

impl<'a> ReceiveRecordBuffer<'a> {
  pub fn new(engine: &'a mut engine::Context) -> ReceiveRecordBuffer {
    ReceiveRecordBuffer { engine }
  }
}

impl<'a> ReadDst for ReceiveRecordBuffer<'a> {
  fn read_from(&mut self, reader: &mut Read) -> Result<usize> {
    let bytes_read = {
      let buffer = self.engine.recvrec_buf()
        .ok_or(Error::new(ErrorKind::Other, "recvrec buffer not available"))?;
      reader.read(buffer)?
    };
    self.engine.recvrec_ack(bytes_read)
      .map(|_| bytes_read)
      .map_err(|_| Error::new(ErrorKind::Other, "engine error after ack"))
  }
}

pub struct SendRecordBuffer<'a> {
  engine: &'a mut engine::Context
}

impl<'a> SendRecordBuffer<'a> {
  pub fn new(engine: &'a mut engine::Context) -> SendRecordBuffer {
    SendRecordBuffer { engine }
  }
}

impl<'a> WriteSrc for SendRecordBuffer<'a> {
  fn write_to(&mut self, writer: &mut Write) -> Result<usize> {
    let bytes_written = {
      let buffer = self.engine.sendrec_buf()
        .ok_or(Error::new(ErrorKind::Other, "sendrec buffer not available"))?;
      writer.write(buffer)?
    };
    self.engine.sendrec_ack(bytes_written)
      .map(|_| bytes_written)
      .map_err(|_| Error::new(ErrorKind::Other, "engine error after ack"))
  }
}
