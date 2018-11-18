use std;
use std::io::Write;
use http;
use io::sources::file;
pub struct StaticDirectoryHandler<'a> {
  root_dir: &'a file::Directory,
  index_file: &'a str
}

impl<'a> StaticDirectoryHandler<'a> {
  pub fn new(root_dir: &'a file::Directory, index_file: &'a str) -> StaticDirectoryHandler<'a> {
    StaticDirectoryHandler {
      root_dir,
      index_file
    }
  }

  fn file_path(&self, request_url: &'a str) -> std::io::Result<file::RelativePath<'a, 'a>> {
    let relative_url = request_url.get(1..).and_then(|relative_url| {
      if relative_url.is_empty() {
        None
      } else {
        Some(relative_url)
      }
    });
    let path = if let Some(relative_url) = relative_url {
      if relative_url.ends_with("/") {
        self.root_dir.sub_dir_with_file(relative_url, self.index_file)
      } else {
        self.root_dir.sub_path(relative_url)
      }
    } else {
      self.root_dir.sub_path(self.index_file)
    }?;
    Ok(path)
  }
}

impl<'a> http::RequestHandler for StaticDirectoryHandler <'a> {
  fn read_headers(&mut self, request: &http::Request, res: &http::Responder) -> std::io::Result<Option<http::Response>> {
    // TODO: use content_range header
    let path = self.file_path(request.url())?;
    let (reader, content_hash_fields) = file::Reader::open(&path, None)?;
    let etag = FileETag::from_content_hash_fields(&content_hash_fields)?;

    if let Some(http::headers::ETagMatch::ETag(etag_to_match)) = request.headers().if_none_match {
      if etag.as_str() == etag_to_match {
        let mut response = res.respond(http::status::NOT_MODIFIED)?;
        response.set_header_writer("ETag", |value| write!(value, "\"{}\"", etag.as_str()))?;
        return Ok(Some(response.into_body()?.finish()));
      }
    }
    
    let mut response = res.respond(http::status::OK)?;
    let mime_type = http::mime_type::from_path(request.url(), Some(self.index_file));
    response.set_header("Content-Type", mime_type)?;
    response.set_header_usize("Content-Length", reader.request_size()?)?;
    response.set_header_writer("ETag", |value| write!(value, "\"{}\"", etag.as_str()))?;
    response.finish_with_file(reader)
  }
}

const FILE_ETAG_SIZE : usize = (16*4) + 3; //3 dashes

struct FileETag {
  buffer: [u8; FILE_ETAG_SIZE],
  len: usize
}

impl FileETag {
  pub fn from_content_hash_fields(fields: &file::ContentHashFields) -> std::io::Result<FileETag> {
    let mut buffer : [u8; FILE_ETAG_SIZE] = unsafe {
      std::mem::uninitialized()
    };
    let pos = {
      let mut etag_writer : &mut [u8] = &mut buffer;
      let mtime = fields.mtime.saturating_mul(1000).saturating_add(fields.mtime_nsec / 1_000_000);
      write!(etag_writer, "{:x}-{:x}-{:x}", fields.inode, fields.size, mtime)?;
      etag_writer.as_ptr() as usize
    };
    let offset = (&buffer).as_ptr() as usize;
    let len = pos - offset;
    Ok(FileETag { len, buffer })
  }

  pub fn as_str(&self) -> &str {
    unsafe {
      std::str::from_utf8_unchecked(&self.buffer[.. self.len])
    }
  }
}
/*
#[cfg(test)]
mod tests {
  #[test]
  fn test_etag() {
    let fields = file::ContentHashFields {
      inode: 0xABCDE,
      size: 0xFFFF,

    }
  }
}
*/
