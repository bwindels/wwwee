use super::wrapper::{engine};
use std::io::{Read, Write, Result, Error, ErrorKind};
use io::{WriteSrc, ReadDst};

pub struct ReceiveRecordChannel<'a> {
  engine: &'a mut engine::Context
}

impl<'a> ReceiveRecordChannel<'a> {
  pub fn new(engine: &'a mut engine::Context) -> ReceiveRecordChannel {
    ReceiveRecordChannel { engine }
  }
}

impl<'a> ReadDst for ReceiveRecordChannel<'a> {
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

pub struct SendRecordChannel<'a> {
  engine: &'a mut engine::Context
}

impl<'a> SendRecordChannel<'a> {
  pub fn new(engine: &'a mut engine::Context) -> SendRecordChannel {
    SendRecordChannel { engine }
  }
}

impl<'a> WriteSrc for SendRecordChannel<'a> {
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
