 x implement url encoded params parsing
 - work on response code
 	- don't error out when no response created by read_headers,
 		but call read_body once if content-length was set, otherwise error
 	- implement responder, where handlers ask for a buffer
    - implement sending static files as response
     - set TCP_CORK - http://baus.net/on-tcp_cork
 - implement json parser/writer
 - parse/add more headers to common headers
 - implement cross-application state based on configuration dictionary to support things like persistent sqlite handle
 - implement xml parser/writer
 - implement cross-request state
 - security
   - TLS - https://bearssl.org/
   - HSTS
   - do we need CORS? is the implemented in the handler or in the connection?
   - is there any stuff we need to do to prevent XSS?
