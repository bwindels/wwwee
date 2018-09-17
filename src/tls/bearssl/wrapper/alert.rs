use super::ffi::*;

#[derive(Clone, Copy, Debug)]
pub enum Alert {
  CloseNotify,
  UnexpectedMessage,
  BadRecordMac,
  RecordOverflow,
  DecompressionFailure,
  HandshakeFailure,
  BadCertificate,
  UnsupportedCertificate,
  CertificateRevoked,
  CertificateExpired,
  CertificateUnknown,
  IllegalParameter,
  UnknownCa,
  AccessDenied,
  DecodeError,
  DecryptError,
  ProtocolVersion,
  InsufficientSecurity,
  InternalError,
  UserCanceled,
  NoRenegotiation,
  UnsupportedExtension,
  NoApplicationProtocol,
  Other(u8)
}

impl Alert {
  pub fn from_primitive(msg: u32) -> Alert {
    match msg {
      BR_ALERT_CLOSE_NOTIFY => Alert::CloseNotify,
      BR_ALERT_UNEXPECTED_MESSAGE => Alert::UnexpectedMessage,
      BR_ALERT_BAD_RECORD_MAC => Alert::BadRecordMac,
      BR_ALERT_RECORD_OVERFLOW => Alert::RecordOverflow,
      BR_ALERT_DECOMPRESSION_FAILURE => Alert::DecompressionFailure,
      BR_ALERT_HANDSHAKE_FAILURE => Alert::HandshakeFailure,
      BR_ALERT_BAD_CERTIFICATE => Alert::BadCertificate,
      BR_ALERT_UNSUPPORTED_CERTIFICATE => Alert::UnsupportedCertificate,
      BR_ALERT_CERTIFICATE_REVOKED => Alert::CertificateRevoked,
      BR_ALERT_CERTIFICATE_EXPIRED => Alert::CertificateExpired,
      BR_ALERT_CERTIFICATE_UNKNOWN => Alert::CertificateUnknown,
      BR_ALERT_ILLEGAL_PARAMETER => Alert::IllegalParameter,
      BR_ALERT_UNKNOWN_CA => Alert::UnknownCa,
      BR_ALERT_ACCESS_DENIED => Alert::AccessDenied,
      BR_ALERT_DECODE_ERROR => Alert::DecodeError,
      BR_ALERT_DECRYPT_ERROR => Alert::DecryptError,
      BR_ALERT_PROTOCOL_VERSION => Alert::ProtocolVersion,
      BR_ALERT_INSUFFICIENT_SECURITY => Alert::InsufficientSecurity,
      BR_ALERT_INTERNAL_ERROR => Alert::InternalError,
      BR_ALERT_USER_CANCELED => Alert::UserCanceled,
      BR_ALERT_NO_RENEGOTIATION => Alert::NoRenegotiation,
      BR_ALERT_UNSUPPORTED_EXTENSION => Alert::UnsupportedExtension,
      BR_ALERT_NO_APPLICATION_PROTOCOL => Alert::NoApplicationProtocol,
      other_msg => Alert::Other(other_msg as u8),
    }
  }
}
