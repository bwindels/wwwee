 - [x] implement url encoded params parsing
 - [ ] work on response code, implement this side by side with current way of handling request, so we always keep the hello world app working

    - [x] move old code to old module
        - Connection
        - ConnectionHandler
        - http::ConnectionHandler
    - [x] implement buffer management
    - [ ] write second Server impl that
          - splits Token in (ConnectionToken, AsyncSourceToken)
          - uses ConnectionToken as index in array of Option<H: io::Handler>
          - passes Context wrapper around Poll and only expose AsyncSourceToken to add more fd's

    - [ ] implement RequestResponseConnection
            RequestResponseConnection has an
            enum State {
              Request(Q: io::Handler<S: io::Handler>)
              Response(S: io::Handler)
            }
            - while reading response keeps track whether socket became readable
            - reregister for only writeable events on AsyncSourceToken 0 after switching to response?
            - doesn't do any actual I/O, just forwards
    - [ ] implement buffer responder as io::Handler<()>
    - [ ] implement http request handler as io::Handler<()>

    At this point, use new server/handlers from main.rs, and remove old module

    - [ ] implement io::file::Reader
    - [ ] implement file responder as io::Handler<()> using io::file::Reader
      - set TCP_CORK on socket - http://baus.net/on-tcp_cork
    - [ ] don't error out when no response created by read_headers,
      but call read_body once if content-length was set, otherwise error

 - [ ] implement json parser/writer
 - [ ] parse/add more headers to common headers
 	- [ ] implement base64 parser (also support base64url) for basic auth
 - [ ] implement cross-request state
 - [ ] implement cross-application state based on configuration dictionary to support things like persistent sqlite handle
 - security
   - [ ] TLS - https://bearssl.org/
      - [ ] first locally stored certificates
      - [ ] lets encrypt supported in server, implement ACME protocol
        - [ ] working https client for this to call lets encrypt api
          - [ ] need http client support
          - [ ] need working ca list to work with bearssl
   - [ ] HSTS
   - [ ] do we need CORS? is the implemented in the handler or in the connection?
   - is there any stuff we need to do to prevent XSS?
 - [ ] implement xml parser/writer
 - [ ] implement file upload by streaming to disk
