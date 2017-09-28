The goal is to write a fast, robust http server that doesn't allocate any dynamic memory.

We'll support a fixed number of connections, with fixed sized buffers for each connection.
We'll handle all connections on one thread, using mio for async I/O

# Limitations

To simplify things, we'll require several things:
- all the request headers fit in the receive buffer
- the request body fits in the receive buffer
	- later on we might add support for streaming resources to disk

If this fails, we respond with `413 Payload Too Large`.

# Parsing headers

We'll first read from the socket until we find `\r\n\r\n`, indicating the end of the headers.
We keep filling the receive buffer until we find this or until it's full (in which case we respond with `413`).
Once we've found the end of the headers, we parse them.

# Reading body

If the application decides that based on the headers, we might have a body, we proceed as follows:
move the bytes that were read past the end of the headers (so the start of the body that was already read) to the beginning of the receive buffer, then we try to read the body after that. if the rest of the body doesn't fit in the rest of the receive body, we reply with `413`.
Once we have the body, we pass it to the application.

# Responding

The application can send a response through filling it's own buffer and writing that, using sendfile, ...

# JSON

We'll also write a stack based JSON parser, that we can use to parse the body with once it's read in