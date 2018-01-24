The goal is to write a fast, robust http server that uses very few resources.

The server won't allocate any dynamic memory. Everything is on the stack.
We'll support a fixed number of connections, with fixed sized buffers for each connection.
We'll handle all connections on one thread, using mio for async I/O

The server is to be used for running a home server on very low-end hardware, so a low amount of concurrent connections shouldn't be a problem.

# Limitations

To simplify things, we'll require several things:
- all the request headers fit in the receive buffer
- the request body fits in the receive buffer
	- later on we might add support for streaming resources to disk

If this fails, we respond with `413 Payload Too Large`.

# Parsing headers

We'll first read from the socket until we find `\r\n\r\n`, indicating the end of the headers.
We keep filling the receive buffer until we find this or until it's full (in which case we respond with `413`).
Once we've found the end of the headers, we parse them assuming utf8. If this fails, we respond with `400 Bad Request`.

# Reading body

If the application decides that based on the headers, we might have a body, we proceed as follows:
move the bytes that were read past the end of the headers (so the start of the body that was already read) to the beginning of the receive buffer, then we try to read the body after that. if the rest of the body doesn't fit in the rest of the receive body, we reply with `413`.
Once we have the body, we pass it to the application.

# Responding

The application can send a response through filling it's own buffer and passing that to the responder, that will write a bit every time the socket becomes writeable, or using sendfile.

Right now everything revolves around the Connection struct. But if we have determined what the response should be to a request, we need to keep track of the socket to write the response, but we won't use the read buffer at that time. We could move the socket to a pool of Responser structs, one for buffer Responders, and one for static file Responders (using sendfile). We could have more responders than Connections. Especially static files we can serve cheaply. So once a connection has returned it's responder, we put the responder data, together with the socket in a responder, and mark the connection as free.

## TLS & sendfile

If we want to use sendfile and TLS together, we'll need to tie ourselves to linux only and use kernel TLS encryption. https://blog.filippo.io/playing-with-kernel-tls-in-linux-4-13-and-go/

# JSON

We'll also write a stack based JSON parser, that we can use to parse the body with once it's read in

# Too many connections

We'll have a preallocated pool of connections. If we run out, we respond with `429 Too Many Requests`.

# Open problems
- How to deal with URL encoding? Decode in place?
	- have separate fixed buffer to decode the url in 
- How to pass response buffer from HttpHandler to Handler?

# Implementation

- use `uhttp_media_type` to parse content-type/accept headers

# Optimization ideas
 - can we use readv with 1 or 2 iovecs so we don't have to copy over data to the beginning after comsuming part of the read buffer (like processing headers)? Would be hard to adapt our code to this because we couldn't process the read result as one slice, which is a hard assumption now in our code.
