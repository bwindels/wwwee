use super::ffi::*;
use std::marker::PhantomData;

pub struct Certificate<'a> {
  cert: br_x509_certificate,
  lt: PhantomData<&'a u8>
}

impl<'a> Certificate<'a> {
  pub fn from_bytes(certificate: &[u8]) -> Certificate<'a> {
    println!("certificate at offset {:x} with len {}", certificate.as_ptr() as usize, certificate.len());
    Certificate {
      cert: br_x509_certificate {
        data: certificate.as_ptr() as *mut u8,
        data_len: certificate.len()
      },
      lt: PhantomData
    }
  }

  pub fn as_ptr(&self) -> *const br_x509_certificate {
    &self.cert as *const br_x509_certificate
  }
}

#[derive(Debug)]
pub enum Error {
  InvalidValue = BR_ERR_X509_INVALID_VALUE as isize,
  Truncated = BR_ERR_X509_TRUNCATED as isize,
  EmptyChain = BR_ERR_X509_EMPTY_CHAIN as isize,
  InnerTrunc = BR_ERR_X509_INNER_TRUNC as isize,
  BadTagClass = BR_ERR_X509_BAD_TAG_CLASS as isize,
  BadTagValue = BR_ERR_X509_BAD_TAG_VALUE as isize,
  IndefiniteLength = BR_ERR_X509_INDEFINITE_LENGTH as isize,
  ExtraElement = BR_ERR_X509_EXTRA_ELEMENT as isize,
  Unexpected = BR_ERR_X509_UNEXPECTED as isize,
  NotConstructed = BR_ERR_X509_NOT_CONSTRUCTED as isize,
  NotPrimitive = BR_ERR_X509_NOT_PRIMITIVE as isize,
  PartialByte = BR_ERR_X509_PARTIAL_BYTE as isize,
  BadBoolean = BR_ERR_X509_BAD_BOOLEAN as isize,
  Overflow = BR_ERR_X509_OVERFLOW as isize,
  BadDn = BR_ERR_X509_BAD_DN as isize,
  BadTime = BR_ERR_X509_BAD_TIME as isize,
  Unsupported = BR_ERR_X509_UNSUPPORTED as isize,
  LimitExceeded = BR_ERR_X509_LIMIT_EXCEEDED as isize,
  WrongKeyType = BR_ERR_X509_WRONG_KEY_TYPE as isize,
  BadSignature = BR_ERR_X509_BAD_SIGNATURE as isize,
  TimeUnknown = BR_ERR_X509_TIME_UNKNOWN as isize,
  Expired = BR_ERR_X509_EXPIRED as isize,
  DnMismatch = BR_ERR_X509_DN_MISMATCH as isize,
  BadServerName = BR_ERR_X509_BAD_SERVER_NAME as isize,
  CriticalExtension = BR_ERR_X509_CRITICAL_EXTENSION as isize,
  NotCa = BR_ERR_X509_NOT_CA as isize,
  ForbiddenKeyUsage = BR_ERR_X509_FORBIDDEN_KEY_USAGE as isize,
  WeakPublicKey = BR_ERR_X509_WEAK_PUBLIC_KEY as isize,
  NotTrusted = BR_ERR_X509_NOT_TRUSTED as isize,
}


