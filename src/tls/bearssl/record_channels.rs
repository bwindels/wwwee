pub struct ReceiveRecordBuffer<'a> {
  ctx: &mut 'a ffi::br_ssl_server_context,
  buffer: &mut 'a [u8]
}

impl<'a> ReadDst for ReceiveRecordBuffer<'a> {
  fn read_from<R: io::Read>(self, reader: &mut R) -> io::Result<usize> {
    reader.read(buffer).map(|bytes_read| {
      ffi::br_ssl_engine_sendrec_ack(self.ctx, bytes_read);
      bytes_read
    })
  }
}

pub struct SendRecordBuffer<'a> {
  ctx: &mut 'a ffi::br_ssl_server_context,
  buffer: &mut 'a [u8]
}

impl<'a> WriteSrc for SendRecordBuffer<'a> {
  fn write_to<R: io::Write>(self, writer: &mut W) -> io::Result<usize> {
    writer.write(slice).map(|bytes_written| {
      ffi::br_ssl_engine_sendrec_ack(self.ctx, bytes_written);
      bytes_written
    });
  }
}
