
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

pub enum Error {
  InvalidValue = ffi::BR_ERR_X509_INVALID_VALUE,
  Truncated = ffi::BR_ERR_X509_TRUNCATED,
  EmptyChain = ffi::BR_ERR_X509_EMPTY_CHAIN,
  InnerTrunc = ffi::BR_ERR_X509_INNER_TRUNC,
  BadTagClass = ffi::BR_ERR_X509_BAD_TAG_CLASS,
  BadTagValue = ffi::BR_ERR_X509_BAD_TAG_VALUE,
  IndefiniteLength = ffi::BR_ERR_X509_INDEFINITE_LENGTH,
  ExtraElement = ffi::BR_ERR_X509_EXTRA_ELEMENT,
  Unexpected = ffi::BR_ERR_X509_UNEXPECTED,
  NotConstructed = ffi::BR_ERR_X509_NOT_CONSTRUCTED,
  NotPrimitive = ffi::BR_ERR_X509_NOT_PRIMITIVE,
  PartialByte = ffi::BR_ERR_X509_PARTIAL_BYTE,
  BadBoolean = ffi::BR_ERR_X509_BAD_BOOLEAN,
  Overflow = ffi::BR_ERR_X509_OVERFLOW,
  BadDn = ffi::BR_ERR_X509_BAD_DN,
  BadTime = ffi::BR_ERR_X509_BAD_TIME,
  Unsupported = ffi::BR_ERR_X509_UNSUPPORTED,
  LimitExceeded = ffi::BR_ERR_X509_LIMIT_EXCEEDED,
  WrongKeyType = ffi::BR_ERR_X509_WRONG_KEY_TYPE,
  BadSignature = ffi::BR_ERR_X509_BAD_SIGNATURE,
  TimeUnknown = ffi::BR_ERR_X509_TIME_UNKNOWN,
  Expired = ffi::BR_ERR_X509_EXPIRED,
  DnMismatch = ffi::BR_ERR_X509_DN_MISMATCH,
  BadServerName = ffi::BR_ERR_X509_BAD_SERVER_NAME,
  CriticalExtension = ffi::BR_ERR_X509_CRITICAL_EXTENSION,
  NotCa = ffi::BR_ERR_X509_NOT_CA,
  ForbiddenKeyUsage = ffi::BR_ERR_X509_FORBIDDEN_KEY_USAGE,
  WeakPublicKey = ffi::BR_ERR_X509_WEAK_PUBLIC_KEY,
  NotTrusted = ffi::BR_ERR_X509_NOT_TRUSTED,
}
