# Wwwee

Wwwee is a web application server optimized for resource-constrained hardware (think ARM SoCs), focusing on low memory usage (aiming to avoid heap allocations completely), robustness, low administrative burden, and an easy API to develop applications with. It is written in [Rust](https://www.rust-lang.org/).

Once the generic server code itself is complete enough, the plan is to develop multiple applications on top, such as a Drive (WebDAV), Calendar (CalDAV), Contacts (CardDAV) and Chat/IM ([Matrix](https://matrix.org)), Micro-blogging ([Mastodon](https://mastodon.social)) and of course a publishing a website/blog, and offer this as a comprehensive suite to self-host most of your internet service from your own home internet connection on affordable and power-efficient hardware, with minimal maintenance and setup.

## FAQ

### Doesn't Nextcloud/Sandstorm.io already do this?

The goals are quite similar, and while these projects offer great value (also because you can use them today), but in my experience require more beefy hardware (in Sandstorms case x86) to work well. Beefier hardware uses more power, is noisier, bigger, and more expensive to buy and run, making self-hosting less accessible to most people.

### Who's working on this?

Just [me](https://github.com/bwindels) for now, but let me know if you want to contribute!

### Is this running somewhere already?

My Raspberry Pi B+ is serving [www.windels.cloud](http://www.windels.cloud) using wwwee, from my home internet connection.

### Wwwee?

The name comes from www and [wee](https://www.merriam-webster.com/dictionary/wee) (as in small). I pronounce it just like wee.

## Status

See also [the more detailed TODO list](doc/TODO.md).

### Done

 - http request parsing
 - single threaded event loop for all I/O
 - buffered responses
 - async file responses

### In progress

 - [TLS support](https://github.com/bwindels/wwwee/commits/tls) using [BearSSL](https://bearssl.org/).
 - [Zero-allocation JSON parsing](https://github.com/bwindels/json-parser-noalloc-rs)
 - [In-place base64 decoder](https://gist.github.com/bwindels/777a1b5b13cd54bcd67dca3c925ca7bb)

### Planned

 - Let's Encrypt support for automatic TLS setup.
 - Share state between requests, like open database connections
 - Improve TCP handling (TCP_CORK, change socket buffer sizes depending on use case, ...)
 - A work queue for things that are too slow to handle in the request loop.
 - a timer, so long-polling requests can timeout
 - let handlers send responses on other open requests,
   for message broadcasting on long-polling requests.

### Further away plans

 - UPnP support for auto-setup port-forwarding.
 - Find solution for when Hairpin-NAT is unavailable.
 - Install apps from central repository ("app store")
 - autoupdate server software
 - precompile complete packages including OS, so it's easy to install. 

## Code examples

### Echo parsed request

```rust
use wwwee::http;
use std::io::Write;
use std::io;

pub struct HelloWorld {}

impl http::RequestHandler for HelloWorld {
  fn read_headers(&mut self, req: &http::Request, responder: &http::Responder) -> io::Result<Option<http::Response>> {
    let mut resp = responder.respond(http::status::OK)?;
    resp.set_header("Content-Type", "text/html")?;
    let mut body = resp.into_body()?;
    write!(body, "<!DOCTYPE html><html><head><meta charset=\"utf-8\"/></head><body>")?;
    write!(body, "<h1>Hello World!</h1>")?;
    write!(body, "<p>You requested: <code>{} {}</code></p>", req.method(), req.url())?;
    write!(body, "<p>Query parameters:</p>")?;
    write!(body, "<ul>")?;
    for p in req.query_params() {
      write!(body, "<li><code>\"{}\"</code> = <code>\"{}\"</code></li>", p.name, p.value)?;
    }
    write!(body, "</ul>")?;
    if let Some(host) = req.headers().host {
      write!(body, "<p>With host: <code>{}</code></p>\n", host)?;
    }
    write!(body, "</body></html>")?;
    Ok(Some(body.finish()))
  }
}
```

### Static file response

```rust
use wwwee::http;
use io::sources::file;
use std;
use std::io::Write;

pub struct StaticFileHandler<'a> {
  path: &'a str,
  content_type: &'a str,
  download_as: Option<&'a str>
}

impl<'a> http::RequestHandler for StaticFileHandler <'a> {
  fn read_headers(&mut self, _request: &http::Request, res: &http::Responder) -> std::io::Result<Option<http::Response>> {
    let path = std::path::Path::new(self.path);
    let reader = file::Reader::open(path, None)?;
    let mut response = res.respond(http::status::OK)?;
    response.set_header("Content-Type", self.content_type)?;
    response.set_header_usize("Content-Length", reader.request_size()?)?;
    if let Some(filename) = self.download_as {
      response.set_header_writer("Content-Disposition", |ref mut value| {
        write!(value, "attachment; filename={}", filename)
      })?;
    }
    response.finish_with_file(reader)
  }
}

```
