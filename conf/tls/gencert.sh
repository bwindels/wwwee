# generate private key
openssl genpkey -algorithm RSA -pkeyopt rsa_keygen_bits:2048 -out private_key.pem
# generate CSR
openssl req -new -config csr.conf -key private_key.pem -out csr.pem
# self-sign CSR into X509 certificate
openssl x509 -req -signkey private_key.pem -in csr.pem -out cert.pem
# convert to DER
openssl x509 -inform PEM -in cert.pem -outform DER -out cert.der
openssl rsa -inform PEM -in private_key.pem -outform DER -out private_key.der
# cleanup
rm private_key.pem cert.pem csr.pem
