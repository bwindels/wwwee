use std::cmp;
use std::ops::{Range};
use super::{bytes_as_block_offset, bytes_as_block_count};

/* helper struct to calculate request range
 * for direct IO on linux which requires 
 * request offsets and sizes to be block aligned:
 *
 *                block aligned
 *         read operations with > +----+----+----+----+----+----+----+
 *  buffer capacity of 2 blocks   |         |         |         |    |
 *                                | +------------------------------+ |
 *        total requested range > | |       |         |         |  | |
 *                     in bytes   | +------------------------------+ |
 *                                +----+----+----+----+----+----+----+
 *                        block > |    |    |    |    |    |    |    |
 *                    alignment
 *         (4096 bytes usually)
 */

pub struct ReadRangeConfig {
  total_byte_range: Range<usize>,
  block_size: u16,
  buffer_block_capacity: u16,  
}

impl ReadRangeConfig {
  pub fn new(total_byte_range: Range<usize>, block_size: u16, buffer_block_capacity: u16) -> ReadRangeConfig {
    ReadRangeConfig {
      total_byte_range,
      block_size,
      buffer_block_capacity
    }
  }

  pub fn buffer_size(&self) -> usize {
    self.block_size as usize * self.buffer_block_capacity as usize
  } 

  #[cfg(test)]
  pub fn block_size(&self) -> usize {
    self.block_size as usize
  }

  pub fn total_range(&self) -> Range<usize> {
    self.total_byte_range.clone()
  }

  pub fn first_range(&self) -> Option<ReadRange> {
    if self.total_byte_range.start == self.total_byte_range.end {
      None
    }
    else {
      let block_offset = bytes_as_block_offset(self.total_byte_range.start, self.block_size);
      Some(ReadRange {
        total_byte_range: self.total_byte_range.clone(),
        block_size: self.block_size,
        buffer_block_capacity: self.buffer_block_capacity,
        block_offset: block_offset
      })
    }
  } 
}

pub struct ReadRange {
  total_byte_range: Range<usize>,
  block_size: u16,
  buffer_block_capacity: u16,
  block_offset: usize
}

impl ReadRange {
  pub fn total_range(&self) -> Range<usize> {
    self.total_byte_range.clone()
  }

  pub fn operation_range(&self) -> Range<usize> {
    let offset = self.block_offset as usize * self.block_size as usize;
    let size = self.block_count() * self.block_size as usize;
    offset .. offset + size
  }

  pub fn buffer_range(&self) -> Range<usize> {
    let op_range = self.operation_range();
    let global_start = cmp::max(op_range.start, self.total_byte_range.start);
    let global_end = cmp::min(op_range.end, self.total_byte_range.end);
    let buffer_range = 
      (global_start - op_range.start) .. 
      (global_end - op_range.start);
    buffer_range
  }

  pub fn next(self) -> Option<ReadRange> {
    if self.operation_range().end >= self.total_byte_range.end {
      None
    }
    else {
      let block_offset = self.block_offset + self.block_count();
      Some(ReadRange {
        total_byte_range: self.total_byte_range,
        block_size: self.block_size,
        buffer_block_capacity: self.buffer_block_capacity,
        block_offset
      })
    }
  }

  fn block_count(&self) -> usize {
    let total_blocks = bytes_as_block_count(self.total_byte_range.end, self.block_size);
    let remaining_blocks = total_blocks - self.block_offset;
    cmp::min(self.buffer_block_capacity as usize, remaining_blocks)
  }
}

#[cfg(test)]
mod tests {
  use super::ReadRangeConfig;

  const C_4096 : usize = 4096;
  const C_8192 : usize = C_4096 * 2;
  const C_12288 : usize = C_4096 * 3;
  const C_16384 : usize = C_4096 * 4;
  const C_20480 : usize = C_4096 * 5;
  const C_24576 : usize = C_4096 * 6;
  const C_32768 : usize = C_4096 * 8;

  #[test]
  fn test_non_aligned_begin_multi_request() {
    let r = ReadRangeConfig::new(100 .. 10_000, C_4096 as u16, 2u16)
      .first_range().unwrap();
    assert_eq!(r.operation_range(), 0 .. C_8192);
    assert_eq!(r.buffer_range(), 100 .. C_8192);
    let r = r.next().expect("range should have a second request");
    assert_eq!(r.operation_range(), C_8192 .. C_12288);
    assert_eq!(r.buffer_range(), 0 .. (10_000 - C_8192));
    assert!(r.next().is_none());
  }

  #[test]
  fn test_non_aligned_long_multi_request() {
    let r = ReadRangeConfig::new(100 .. 20_000, C_4096 as u16, 2u16)
      .first_range().unwrap();
    assert_eq!(r.operation_range(), 0 .. C_8192);
    assert_eq!(r.buffer_range(), 100 .. C_8192);
    let r = r.next().expect("range should have a second request");
    assert_eq!(r.operation_range(), C_8192 .. C_16384);
    assert_eq!(r.buffer_range(), 0 .. C_8192);
    let r = r.next().expect("range should have a third request");
    //only 1 block since second block would be ignored
    assert_eq!(r.operation_range(), C_16384 .. C_20480);
    assert_eq!(r.buffer_range(), 0 .. 20_000 - C_16384);
    assert!(r.next().is_none());
  }

  #[test]
  fn test_non_aligned_mid_multi_request() {
    let r = ReadRangeConfig::new(20_000 .. 30_000, C_4096 as u16, 2u16)
      .first_range().unwrap();
    assert_eq!(r.operation_range(), C_16384 .. C_24576);
    assert_eq!(r.buffer_range(), 20_000 - C_16384 .. C_8192);
    let r = r.next().expect("range should have a second request");
    assert_eq!(r.operation_range(), C_24576 .. C_32768);
    assert_eq!(r.buffer_range(), 0 .. (30_000 - C_24576));
    assert!(r.next().is_none());
  }

  #[test]
  fn test_end_aligned_single_request() {
    let r = ReadRangeConfig::new(100 .. C_8192, C_4096 as u16, 2u16)
      .first_range().unwrap();
    assert_eq!(r.operation_range(), 0 .. C_8192);
    assert_eq!(r.buffer_range(), 100 .. C_8192);
    assert!(r.next().is_none());
  }

  #[test]
  fn test_start_aligned_single_request() {
    let r = ReadRangeConfig::new(0 .. 8_000, C_4096 as u16, 2u16)
      .first_range().unwrap();
    assert_eq!(r.operation_range(), 0 .. C_8192);
    assert_eq!(r.buffer_range(), 0 .. 8_000);
    assert!(r.next().is_none());
  }

  #[test]
  fn test_start_aligned_single_small_request() {
    let r = ReadRangeConfig::new(0 .. 4_000, C_4096 as u16, 2u16)
      .first_range().unwrap();
    assert_eq!(r.operation_range(), 0 .. C_4096);
    assert_eq!(r.buffer_range(), 0 .. 4_000);
    assert!(r.next().is_none());
  }
}
