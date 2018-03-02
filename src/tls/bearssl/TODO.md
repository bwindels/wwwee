# Roadmap to TLS support

## Shortlist:
  2. write skey decoder function
  3. implement context
  4. implement rec channels
  5. implement socket
  6. implement Buffer::read_from_with_hint

## Hardcoded, self-signed certificate

First thing to support is a hardcoded, self-signed certificate. Just old-school RSA, no elliptic curve. 


### Write TLSContext helper
  - make PageBuffer api public
  - allocate PageBuffer of size BR_SSL_BUFSIZE_BIDI
  - TLSContext contains the PageBuffer so it doesn't have to borrow the buffer to pass it to brssl
  - check error after every input and drop socket on error

### Write TLSHandler

It's an io::Handler that has a child io::Handler to forward events to it's child with decoded data. It owns the socket, since we want to read the socket straight into it's `recvrec` buffer.

Since the TLS recvapp buffer is only temporarily available we need to copy anyway, so TLSContext/Handler can expose a standard Read for the decrypted socket.

I was thinking of setting the socket buffer size to the TLS record size, but that might not be optimal. If we can't read all the socket data into the recvrec buffer in one go, we'll just need to write several times because the data will be decrypted and on the application side be appended to a request-scoped buffer. For the case when that doesn't happen we might have to change the socket trigger to level triggered so we get events when there is still data in the socket?... this would be a weird scenario because the app should always straight away respond to events. If it doesn't read the decrypted data straight away that would be a bug almost.


All handlers above TLSHandler will assume AsyncToken(0) refers to an event with a borrowed source.

## Generate a certificate on server start-up

And for now sign it ourselves.

The pragmatic way here will be to call `openssl` with `exec`. We could also link with OpenSSL, just for this, but that would probably be more work. BearSSL can't generate keys or sign certificates. The whole process (including SANs) is (well described)[http://apetec.com/support/GenerateSAN-CSR.htm]. Would be like this:

  - `openssl genpkey -algorithm RSA -pkeyopt rsa_keygen_bits:2048`
  - `<private key> | openssl rsa -pubout`
  - generate CSR, first writing a config file, and then calling `openssl req -new ...`.
  - create X509 cert by signing the CSR with our own private key: `openssl x509 -req ...`

## Support ACME and *Let's Encrypt*

we'll need the following components before we can start on this:

  - base64(url) encoder (could borrow a Write impl and impl Write itself and encode on the fly)
  - a JSON writer
  - a JSON parser
  - an (async) http client to use inside the server, with tls support (with at least letsencrypt as thrust anchor)

by the time we start on this, ACME v2 should have been rolled out for production use (Q1 2018).
Lets encrypt will only support EC signatures in Q3 2018.
