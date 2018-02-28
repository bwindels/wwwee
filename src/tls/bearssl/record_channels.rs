use super::ffi;
use std::io::{Read, Write, Result, Error, ErrorKind};
use io::{WriteSrc, ReadDst};

pub struct ReceiveRecordBuffer<'a> {
  eng: &mut 'a ffi::br_ssl_engine_context,
  buffer: Option<&mut 'a [u8]>
}

impl<'a> ReadDst for ReceiveRecordBuffer<'a> {
  fn read_from<R: Read>(&mut self, reader: &mut R) -> Result<usize> {
    let buffer = self.buffer.ok_or(Error::new(
      ErrorKind::Other,
      "tls recvrec buffer was consumed already"))?;
    reader.read(buffer).map(|bytes_read| {
      ffi::br_ssl_engine_recvrec_ack(self.eng, bytes_read);
      self.buffer = None;
      bytes_read
    })
  }
}

pub struct SendRecordBuffer<'a> {
  eng: &mut 'a ffi::br_ssl_engine_context,
  buffer: Option<&mut 'a [u8]>
}

impl<'a> WriteSrc for SendRecordBuffer<'a> {
  fn write_to<R: Write>(&mut self, writer: &mut W) -> Result<usize> {
    let buffer = self.buffer.ok_or(Error::new(
      ErrorKind::Other,
      "tls sendrec buffer was consumed already"))?;
    writer.write(buffer).map(|bytes_written| {
      ffi::br_ssl_engine_sendrec_ack(self.eng, bytes_written);
      self.buffer = None;
      bytes_written
    });
  }
}
