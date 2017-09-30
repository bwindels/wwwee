pub enum Authorization<'a> {
  Basic(&'a str),
  Bearer(&'a str)
}
