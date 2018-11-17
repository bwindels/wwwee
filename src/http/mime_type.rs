pub fn from_path(url: &str, default_filename: Option<&str>) -> &'static str {
  match extension_from_path(url, default_filename) {
    Some("htm") |
    Some("html") => "text/html",
    Some("txt") => "text/plain",
    Some("css") => "text/css",
    Some("md") => "text/markdown",
    Some("jpg") |
    Some("jpeg") => "image/jpeg",
    Some("png") => "image/png",
    Some("gif") => "image/gif",
    Some("js") |
    Some("mjs") => "application/javascript",
    Some("xml") => "application/xml",
    _ => "application/octet-stream"
  }
}

fn extension_from_path<'a>(url: &'a str, default_filename: Option<&'a str>) -> Option<&'a str> {
  let filename = url.rfind('/').and_then(|filename_start| {
    url.get(filename_start + 1 ..)
  }).and_then(|filename| {
    if filename.is_empty() { None } else { Some(filename) }
  }).or(default_filename);

  let extension = filename.and_then(|f| {
    f.rfind('.').and_then(|extension_start| {
      f.get(extension_start + 1 ..)
    })
  });
  extension
}

#[cfg(test)]
mod tests {
  use super::from_path;

  const DEFAULT_TYPE : &'static str = "application/octet-stream";

  #[test]
  fn test_from_path() {
    assert_eq!(from_path("/foo.jpg", None), "image/jpeg");
    assert_eq!(from_path("/dir.lala/foo.jpg", None), "image/jpeg");
    assert_eq!(from_path("/", Some("index.html")), "text/html");
    assert_eq!(from_path("/foo", Some("index.html")), DEFAULT_TYPE);
    assert_eq!(from_path("/foo.bar", None), DEFAULT_TYPE);
  }
}
