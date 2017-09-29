pub struct StrSplitMut<'a, 'b> {
  string: Option<&'a mut str>,
  pattern: &'b str,
}

pub fn str_split_mut<'a, 'b>(string: &'a mut str, pattern: &'b str) -> StrSplitMut<'a, 'b> {
  StrSplitMut {
    string: Some(string),
    pattern,
  }
}

impl<'a, 'b> Iterator for StrSplitMut<'a, 'b> {
  type Item = &'a mut str;

  fn next(&mut self) -> Option<Self::Item> {
    let s = match self.string.take() {
      Some(s) => s,
      None => return None,
    };
    if let Some(end_idx) = s.find(self.pattern) {
      let (subslice, remainder) = s.split_at_mut(end_idx);
      self.string = Some(&mut remainder[self.pattern.len()..]);
      Some(subslice)
    }
    else if !s.is_empty() {
      Some(s)
    }
    else {
      None
    }
  }
}
