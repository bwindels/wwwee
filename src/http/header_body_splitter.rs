use std::cmp;

pub struct HeaderBodySplitter {
  find_offset: usize
}

impl HeaderBodySplitter {
  pub fn new() -> HeaderBodySplitter {
    HeaderBodySplitter{find_offset: 1}
  }

  pub fn try_split<'a>(&mut self, buffer: &'a mut [u8]) -> Option<(&'a mut [u8], &'a mut [u8])> {
    const HEADER_END: &'static [u8] = b"\r\n\r\n";
    let offset = cmp::max(HEADER_END.len(), self.find_offset + 1) - HEADER_END.len();
    //update the offset where to look from next update
    self.find_offset = buffer.len();

    buffer.get(offset..).and_then(|search_space| {
      search_space.windows(HEADER_END.len())
        .position(|window| window == HEADER_END)
    })
    .map(|header_end| offset + header_end + HEADER_END.len())
    .map(move |header_end| buffer.split_at_mut(header_end))
    .map(|(headers, body)| {
      let len = headers.len();
      (&mut headers[..len - HEADER_END.len()], body)
    })
  }
}


#[cfg(test)]
mod tests {
  #[test]
  fn test_incremental_split() {
    let mut st = "foobar\r\nhello\r\n\r\nhaha".to_string();
    let mut s = unsafe { st.as_bytes_mut() };
    let mut splitter = super::HeaderBodySplitter::new();
    assert_eq!(splitter.update(&mut s[0..13]), None);
    assert_eq!(splitter.update(&mut s[0..16]), None);
    match splitter.update(&mut s) {
      Some((headers, body)) => {
        assert_eq!(headers, b"foobar\r\nhello");
        assert_eq!(body, b"haha");
      }
      None => panic!("should be Some")
    };
  }

}