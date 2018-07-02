LOC=$(pwd)
cd src/tls/bearssl/lib/BearSSL/
make -f mk/SingleUnix.mk clean
cd $LOC
