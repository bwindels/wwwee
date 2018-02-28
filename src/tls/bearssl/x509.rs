
pub struct X509Certificate {
  cert: ffi::br_x509_certificate
}

impl X509Certificate {
  pub fn new(certificate: &[u8]) -> X509Certificate {
    X509Certificate {
      cert: ffi::br_x509_certificate {
        data: certificate.as_ptr(),
        data_len: certificate.len()
      }
    }
  }
}
