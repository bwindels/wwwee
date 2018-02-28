//we'll use a br_skey_decoder_context here to decode a private key file
//also an enum to support rsa and ec private keys
use super::{ffi, x509};

pub enum SecretKey {
  Rsa(ffi::br_rsa_private_key),
  EllipticCurve(ffi::br_ec_private_key)
}

pub fn decode_private_key_der_format(der_bytes: &[u8]) -> Result<SecretKey, x509::Error> {

}
