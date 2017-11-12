

struct BufferPool {
  
}

enum Error {
  Full
}

impl BufferPool {
  fn new(alignment: usize) -> BufferPool {
    
  }

  fn borrow_buffer<'b>(&'b mut self, _min_size: usize) -> Result<Buffer<'b>, Error> {
    for b in self.buffers.iter() {
      if let Ok(r) = b.try_borrow_mut() {
        return Ok(Buffer::new(r));
      }
    }
    return Err(Error::Full);
  }
}
