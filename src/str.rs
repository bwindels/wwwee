pub trait SplitMutBuffer {
  fn split_at_mut(&mut self, index: usize) -> (&mut Self, &mut Self);
  fn find(&self, pattern: &Self) -> Option<usize>;
  fn len(&self) -> usize;
  fn remove_left(&mut self, index: usize) -> &mut Self;
}

impl SplitMutBuffer for str {
  fn split_at_mut(&mut self, index: usize) -> (&mut Self, &mut Self) {
    self.split_at_mut(index)
  }
  fn find(&self, pattern: &Self) -> Option<usize> {
    self.find(pattern)
  }
  fn len(&self) -> usize {
    self.len()
  }
  fn remove_left(&mut self, index: usize) -> &mut Self {
    &mut self[index ..]
  }
}

impl SplitMutBuffer for [u8] {
  fn split_at_mut(&mut self, index: usize) -> (&mut [u8], &mut [u8]) {
    self.split_at_mut(index)
  }
  fn find(&self, pattern: &[u8]) -> Option<usize> {
    self.windows(pattern.len()).position(|buf| buf == pattern)
  }
  fn len(&self) -> usize {
    self.len()
  }
  fn remove_left(&mut self, index: usize) -> &mut Self {
    &mut self[index ..]
  }
}

pub struct SplitMutIterator<'a, S>
  where S: SplitMutBuffer + ?Sized + 'a
{
  string: Option<&'a mut S>,
  pattern: &'a S,
}

pub fn str_split_mut<'a, S: SplitMutBuffer + 'a>(string: &'a mut S, pattern: &'a S) -> SplitMutIterator<'a, S> 
  where S: SplitMutBuffer + ?Sized + 'a
{
  SplitMutIterator {
    string: Some(string),
    pattern,
  }
}

impl<'a, S> Iterator for SplitMutIterator<'a, S>
  where S: SplitMutBuffer + ?Sized + 'a
{
  type Item = &'a mut S;

  fn next(&mut self) -> Option<Self::Item> {
    let s = match self.string.take() {
      Some(s) => s,
      None => return None,
    };
    if let Some(end_idx) = s.find(self.pattern) {
      let (subslice, remainder) = s.split_at_mut(end_idx);
      let remainder_w_pattern = remainder.remove_left(self.pattern.len());
      self.string = Some(remainder_w_pattern);
      Some(subslice)
    }
    else if s.len() != 0 {
      Some(s)
    }
    else {
      None
    }
  }
}

#[cfg(test)]
mod test {

  use std::str;

  fn copy_str(dst: &mut [u8], src: &[u8]) {
    assert_eq!(src.len(), dst.len());
    let mut src_it = src.iter();
    for mut d in dst {
      *d = *src_it.next().unwrap();
    }
  }

  #[test]
  fn test_byteslice_split() {
    let mut buffer = [0u8;8];
    copy_str(&mut buffer, b"hi ho ha");
    let pattern = [0x20];
    let mut it = super::str_split_mut(buffer.as_mut(), pattern.as_ref());
    //the map turns the mut ref into a ref 
    assert_eq!(it.next().map(|w| &*w), Some(b"hi".as_ref()));
    assert_eq!(it.next().map(|w| &*w), Some(b"ho".as_ref()));
    assert_eq!(it.next().map(|w| &*w), Some(b"ha".as_ref()));
    assert_eq!(it.next().map(|w| &*w), None);
  }

  #[test]
  fn test_str_split() {
    let mut b = [0u8;8];
    copy_str(&mut b, b"hi ho ha");
    let mut s = str::from_utf8_mut(&mut b).unwrap();
    let mut it = super::str_split_mut(s, " ");
    //the map turns the mut ref into a ref 
    assert_eq!(it.next().map(|w| &*w), Some("hi"));
    assert_eq!(it.next().map(|w| &*w), Some("ho"));
    assert_eq!(it.next().map(|w| &*w), Some("ha"));
    assert_eq!(it.next().map(|w| &*w), None);
  }
}
