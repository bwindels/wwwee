scenarios:
	- initial request, [read body], send response
	- initial request, [read body], do operation on other thread, send response
	- initial request, [read body], do async operation in event loop (process file, http request to other server), send response

		we need to control when the read buffer can be deallocated
		we need to be able to receive events from async sources in the http handler

		connection structure has a couple of async sources (sockets, files, notifications from other threads) attached to it

		buffers are independent of the connection structure:
		when we drop the Request object, the read buffer is deallocated
		when we drop the body, the read buffer is deallocated

		buffer management is transparent to rest of server, could be stack allocated with RefCell, or could be Box, or combination

in case of async static file, epoll will come back that a read operation finished, how do we get that information to the FileResponder in question?
	do responders need a reference to the connection struct?
	some async sources will need a reference, like http requests to other server

	we could use the Token as follows:
		it's a WORD sized value, so on a 32bit cpu we take 24bit for the index of the connection struct is responsable for a Evented source, and 8 bits for a source internal to that connection,

			so the socket to the client for a connection would be: 	0x12345600,
			and subsequent async resources would have token:		0x12345601,
																	0x12345602,
																	...
			this means that a connection never needs to be aware of it's index, only about the
			async resource number, with 0 always being the client socket.
			the connection needs access to the buffer allocation and adding async resources

			since usize is WORD sized, we'll need wrapper types to abstract the different split we want to make with different WORD sizes, usize = ComposedIdx = (ConnectionIdx, AsyncSourceIdx).