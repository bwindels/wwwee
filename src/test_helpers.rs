pub fn copy_str(dst: &mut [u8], src: &[u8]) {
  assert_eq!(src.len(), dst.len());
  let mut src_it = src.iter();
  for d in dst {
    *d = *src_it.next().unwrap();
  }
}