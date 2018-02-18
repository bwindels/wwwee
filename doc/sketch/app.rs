enum MountPoint<'a> {
  Subdomain(&'a str),
  RootPath(&'a str)
}

struct Version {
  pub major: u8,
  pub minor: u16,
  pub patch: u8
}

trait App<T> : http::RequestHandler {
  fn new(cfg: AppConfig) -> Result<Self>;
  fn mount_point(&'a self) -> MountPoint<'a>;
  fn version() -> Version;
  fn upgrade(&mut self, old: Version, new: Version) -> io::Result<()>;
  fn shared_request_state(&'a mut self) -> &'a mut T
}
